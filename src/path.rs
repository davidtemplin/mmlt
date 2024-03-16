use std::collections::VecDeque;

use crate::{
    geometry::Geometry,
    image::PixelCoordinates,
    interaction::Interaction,
    ray::Ray,
    sampler::{MmltSampler, Sampler},
    scene::Scene,
    spectrum::Spectrum,
    types::PathType,
    util,
};

#[derive(Debug)]
pub struct Path {
    vertices: Vec<Vertex>,
    technique: Technique,
    pixel_coordinates: PixelCoordinates,
}

#[derive(Debug)]
pub struct Vertex {
    throughput: Spectrum,
    forward_pdf: Option<f64>,
    reverse_pdf: Option<f64>,
}

impl Vertex {
    fn camera_path_weight(&self) -> Option<f64> {
        if self.forward_pdf.is_none() && self.reverse_pdf.is_none() {
            None
        } else {
            let fwd = self.forward_pdf.unwrap_or(1.0);
            let rev = self.reverse_pdf.unwrap_or(1.0);
            if fwd != 0.0 {
                Some(rev / fwd)
            } else {
                Some(rev)
            }
        }
    }

    fn light_path_weight(&self) -> Option<f64> {
        if self.forward_pdf.is_none() && self.reverse_pdf.is_none() {
            None
        } else {
            let fwd = self.forward_pdf.unwrap_or(1.0);
            let rev = self.reverse_pdf.unwrap_or(1.0);
            if rev != 0.0 {
                Some(fwd / rev)
            } else {
                Some(fwd)
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Technique {
    camera: usize,
    light: usize,
}

impl Technique {
    pub fn sample(path_length: usize, sampler: &mut impl Sampler) -> Technique {
        let end = path_length as f64 + 1.0;
        let r = sampler.sample(0.0..end);
        let camera = r.floor() as usize;
        let light = path_length - camera;
        Technique::new(camera, light)
    }

    pub fn new(camera: usize, light: usize) -> Technique {
        Technique { camera, light }
    }

    pub fn path_type(&self, n: usize) -> PathType {
        if n < self.camera {
            PathType::Camera
        } else {
            PathType::Light
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Contribution {
    pub scalar: f64,
    pub spectrum: Spectrum,
    pub pixel_coordinates: PixelCoordinates,
}

impl Contribution {
    pub fn empty() -> Contribution {
        Contribution {
            scalar: 0.0,
            spectrum: Spectrum::black(),
            pixel_coordinates: PixelCoordinates { x: 0, y: 0 },
        }
    }

    pub fn is_empty(&self) -> bool {
        self.scalar == 0.0
    }

    pub fn acceptance(
        current_contribution: Contribution,
        proposal_contribution: Contribution,
    ) -> f64 {
        if current_contribution.scalar > 0.0 {
            f64::max(
                f64::min(
                    1.0,
                    proposal_contribution.scalar / current_contribution.scalar,
                ),
                0.0,
            )
        } else {
            1.0
        }
    }
}

const TECHNIQUE_STREAM: usize = 0;
const LIGHT_STREAM: usize = 1;
const CAMERA_STREAM: usize = 2;
const STREAM_COUNT: usize = 3;

impl<'a> Path {
    pub fn sampler() -> MmltSampler {
        MmltSampler::new(STREAM_COUNT)
    }

    pub fn contribute(
        scene: &Scene,
        sampler: &mut impl Sampler,
        path_length: usize,
    ) -> Contribution {
        if let Some(path) = Path::generate(scene, sampler, path_length) {
            path.contribution()
        } else {
            Contribution::empty()
        }
    }

    pub fn generate(scene: &Scene, sampler: &mut impl Sampler, path_length: usize) -> Option<Path> {
        sampler.start_stream(TECHNIQUE_STREAM);
        let technique = Technique::sample(path_length, sampler);
        if technique.camera == 0 {
            Path::connect_full_light_path(scene, sampler, technique)
        } else if technique.camera == 1 {
            if technique.light == 1 {
                Path::connect_camera_to_light(scene, sampler, technique)
            } else {
                Path::connect_camera_to_light_subpath(scene, sampler, technique)
            }
        } else {
            if technique.light == 0 {
                Path::connect_full_camera_path(scene, sampler, technique)
            } else if technique.light == 1 {
                Path::connect_camera_subpath_to_light(scene, sampler, technique)
            } else {
                Path::connect_camera_subpath_to_light_subpath(scene, sampler, technique)
            }
        }
    }

    fn connect_camera_to_light(
        scene: &Scene,
        sampler: &mut impl Sampler,
        technique: Technique,
    ) -> Option<Path> {
        sampler.start_stream(CAMERA_STREAM);
        let sampled_camera_interaction = scene.camera.sample_interaction(sampler);
        sampler.start_stream(LIGHT_STREAM);
        let light = scene.sample_light(sampler);
        let mut light_interaction = light.sample_interaction(sampler);
        let ray_direction =
            sampled_camera_interaction.geometry().point - light_interaction.geometry().point;
        let ray = Ray::new(light_interaction.geometry().point, ray_direction);
        let camera_interaction = scene.intersect(ray).filter(|i| i.is_camera())?;
        light_interaction.set_direction(-camera_interaction.geometry().direction);
        let mut interactions: VecDeque<Interaction> = VecDeque::new();
        interactions.push_back(camera_interaction);
        interactions.push_back(light_interaction);
        Path::connect(&mut interactions, technique)
    }

    fn connect_full_light_path(
        scene: &Scene,
        sampler: &mut impl Sampler,
        technique: Technique,
    ) -> Option<Path> {
        sampler.start_stream(LIGHT_STREAM);
        let light = scene.sample_light(sampler);
        let light_interaction = light.sample_interaction(sampler);
        let mut interactions = Path::trace(
            scene,
            sampler,
            light_interaction,
            technique.light,
            PathType::Light,
        )?;
        interactions.front().filter(|i| i.is_camera())?;
        Path::connect(&mut interactions, technique)
    }

    fn connect_full_camera_path(
        scene: &Scene,
        sampler: &mut impl Sampler,
        technique: Technique,
    ) -> Option<Path> {
        sampler.start_stream(CAMERA_STREAM);
        let camera_interaction = scene.camera.sample_interaction(sampler);
        let mut interactions = Path::trace(
            scene,
            sampler,
            camera_interaction,
            technique.camera,
            PathType::Camera,
        )?;
        interactions.back().filter(|i| i.is_light())?;
        Path::connect(&mut interactions, technique)
    }

    fn connect_camera_to_light_subpath(
        scene: &Scene,
        sampler: &mut impl Sampler,
        technique: Technique,
    ) -> Option<Path> {
        sampler.start_stream(LIGHT_STREAM);
        let light = scene.sample_light(sampler);
        let light_interaction = light.sample_interaction(sampler);
        let mut interactions = Path::trace(
            scene,
            sampler,
            light_interaction,
            technique.light,
            PathType::Light,
        )?;
        let last = interactions.front().filter(|i| i.is_object())?;
        sampler.start_stream(CAMERA_STREAM);
        let sampled_camera_interaction = scene.camera.sample_interaction(sampler);
        let ray = Ray::new(
            last.geometry().point,
            sampled_camera_interaction.geometry().point - last.geometry().point,
        );
        let camera_interaction = scene.intersect(ray).filter(|i| i.is_camera())?;
        interactions.push_front(camera_interaction);
        Path::connect(&mut interactions, technique)
    }

    fn connect_camera_subpath_to_light(
        scene: &Scene,
        sampler: &mut impl Sampler,
        technique: Technique,
    ) -> Option<Path> {
        sampler.start_stream(CAMERA_STREAM);
        let camera_interaction = scene.camera.sample_interaction(sampler);
        let mut interactions = Path::trace(
            scene,
            sampler,
            camera_interaction,
            technique.camera,
            PathType::Camera,
        )?;
        let last = interactions.back().filter(|i| i.is_object())?;
        sampler.start_stream(LIGHT_STREAM);
        let light = scene.sample_light(sampler);
        let sampled_light_interaction = light.sample_interaction(sampler);
        let ray = Ray::new(
            last.geometry().point,
            sampled_light_interaction.geometry().point - last.geometry().point,
        );
        let light_interaction = scene.intersect(ray).filter(|i| i.is_light())?;
        interactions.push_back(light_interaction);
        Path::connect(&mut interactions, technique)
    }

    fn connect_camera_subpath_to_light_subpath(
        scene: &Scene,
        sampler: &mut impl Sampler,
        technique: Technique,
    ) -> Option<Path> {
        sampler.start_stream(CAMERA_STREAM);
        let camera_interaction = scene.camera.sample_interaction(sampler);
        let camera_interactions = Path::trace(
            scene,
            sampler,
            camera_interaction,
            technique.camera,
            PathType::Camera,
        )?;
        sampler.start_stream(LIGHT_STREAM);
        let light = scene.sample_light(sampler);
        let light_interaction = light.sample_interaction(sampler);
        let light_interactions = Path::trace(
            scene,
            sampler,
            light_interaction,
            technique.light,
            PathType::Light,
        )?;
        let camera_last = camera_interactions.back().filter(|i| i.is_object())?;
        let light_last = light_interactions.front().filter(|i| i.is_object())?;
        let ray = Ray::new(
            camera_last.geometry().point,
            light_last.geometry().point - camera_last.geometry().point,
        );
        scene.intersect(ray).filter(|i| i.id() == light_last.id())?;
        let mut interactions = camera_interactions;
        interactions.extend(light_interactions);
        Path::connect(&mut interactions, technique)
    }

    fn trace(
        scene: &'a Scene,
        sampler: &mut impl Sampler,
        interaction: Interaction<'a>,
        length: usize,
        path_type: PathType,
    ) -> Option<VecDeque<Interaction<'a>>> {
        let mut stack: VecDeque<Interaction<'a>> = VecDeque::new();
        let mut ray = interaction.initial_ray()?;
        match path_type {
            PathType::Camera => stack.push_back(interaction),
            PathType::Light => stack.push_front(interaction),
        };
        for _ in 1..length {
            let interaction = scene.intersect(ray)?;
            ray = interaction.generate_ray(path_type, sampler)?;
            match path_type {
                PathType::Camera => stack.push_back(interaction),
                PathType::Light => stack.push_front(interaction),
            };
        }
        Some(stack)
    }

    fn connect(interactions: &mut VecDeque<Interaction>, technique: Technique) -> Option<Path> {
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut pixel_coordinates: Option<PixelCoordinates> = None;
        let mut area_pdf: Option<f64> = None;
        let mut previous_geometry: Option<Geometry> = None;
        for (index, interaction) in interactions.iter().enumerate() {
            let next_geometry = interactions.get(index + 1).map(Interaction::geometry);
            match interaction {
                Interaction::Camera(camera_interaction) => {
                    pixel_coordinates = Some(camera_interaction.pixel_coordinates);
                    let point = camera_interaction.geometry.point;
                    let direction = next_geometry?.point - point;
                    let importance = camera_interaction.camera.importance(point, direction);
                    let normal = camera_interaction.geometry.normal;
                    let next_normal = next_geometry?.normal;
                    let geometry_term = util::geometry_term(direction, normal, next_normal);
                    let throughput = importance * geometry_term;
                    let positional_pdf = camera_interaction.camera.positional_pdf(point);
                    let directional_pdf = camera_interaction.camera.directional_pdf(direction);
                    area_pdf = directional_pdf
                        .map(|p| p * util::direction_to_area(direction, next_normal));
                    let vertex = match technique.path_type(index) {
                        PathType::Camera => Vertex {
                            throughput,
                            forward_pdf: positional_pdf,
                            reverse_pdf: None,
                        },
                        PathType::Light => Vertex {
                            throughput,
                            forward_pdf: None,
                            reverse_pdf: positional_pdf,
                        },
                    };
                    vertices.push(vertex);
                }
                Interaction::Light(light_interaction) => {
                    let point = light_interaction.geometry.point;
                    let normal = light_interaction.geometry.normal;
                    let direction = previous_geometry?.point - point;
                    let throughput = light_interaction.light.radiance(point, normal, direction);
                    let sampling_pdf = light_interaction.light.sampling_pdf();
                    let positional_pdf = light_interaction.light.positional_pdf(point);
                    let directional_pdf =
                        light_interaction.light.directional_pdf(normal, direction);
                    let vertex = match technique.path_type(index) {
                        PathType::Camera => Vertex {
                            throughput,
                            forward_pdf: area_pdf,
                            reverse_pdf: sampling_pdf
                                .and_then(|p1| positional_pdf.map(|p2| p1 * p2)),
                        },
                        PathType::Light => Vertex {
                            throughput,
                            forward_pdf: sampling_pdf
                                .and_then(|p1| positional_pdf.map(|p2| p1 * p2)),
                            reverse_pdf: area_pdf,
                        },
                    };
                    vertices.push(vertex);
                    let previous_vertex = &mut vertices[index - 1];
                    let previous_normal = previous_geometry?.normal;
                    let direction_to_area = util::direction_to_area(direction, previous_normal);
                    area_pdf = directional_pdf.map(|p| p * direction_to_area);
                    match technique.path_type(index - 1) {
                        PathType::Camera => {
                            previous_vertex.reverse_pdf = area_pdf;
                        }
                        PathType::Light => {
                            previous_vertex.forward_pdf = area_pdf;
                        }
                    }
                }
                Interaction::Object(object_interaction) => {
                    let point = object_interaction.geometry.point;
                    let normal = object_interaction.geometry.normal;
                    let next_normal = next_geometry?.normal;
                    let wo = previous_geometry?.point - point;
                    let wi = next_geometry?.point - point;
                    let reflectance = object_interaction.reflectance(wo, wi);
                    let geometry_term = util::geometry_term(wi, normal, next_normal);
                    let throughput = reflectance * geometry_term;
                    let vertex = match technique.path_type(index) {
                        PathType::Camera => Vertex {
                            throughput,
                            forward_pdf: area_pdf,
                            reverse_pdf: None,
                        },
                        PathType::Light => Vertex {
                            throughput,
                            forward_pdf: None,
                            reverse_pdf: area_pdf,
                        },
                    };
                    vertices.push(vertex);
                    let previous_vertex = &mut vertices[index - 1];
                    let previous_normal = previous_geometry?.normal;
                    let previous_directional_pdf = object_interaction.pdf(wo, wi, PathType::Light);
                    let previous_direction_to_area = util::direction_to_area(wo, previous_normal);
                    let previous_area_pdf =
                        previous_directional_pdf.map(|p| p * previous_direction_to_area);
                    match technique.path_type(index - 1) {
                        PathType::Camera => {
                            previous_vertex.reverse_pdf = previous_area_pdf;
                        }
                        PathType::Light => {
                            previous_vertex.forward_pdf = previous_area_pdf;
                        }
                    }
                    let next_normal = next_geometry?.normal;
                    let next_directional_pdf = object_interaction.pdf(wo, wi, PathType::Camera);
                    let next_direction_to_area = util::direction_to_area(wi, next_normal);
                    area_pdf = next_directional_pdf.map(|p| p * next_direction_to_area);
                }
            }

            previous_geometry = Some(interaction.geometry());
        }

        let path = Path {
            vertices,
            technique,
            pixel_coordinates: pixel_coordinates?,
        };

        Some(path)
    }

    pub fn contribution(&self) -> Contribution {
        let p = self.pdf();
        if p == 0.0 {
            return Contribution::empty();
        }

        let t = self.throughput();
        if t.is_black() {
            return Contribution::empty();
        }

        let w = self.weight();
        if w == 0.0 {
            return Contribution::empty();
        }

        let c = t * w / p;

        Contribution {
            scalar: c.luminance(),
            spectrum: c,
            pixel_coordinates: self.pixel_coordinates,
        }
    }

    pub fn throughput(&self) -> Spectrum {
        self.vertices
            .iter()
            .map(|v| v.throughput)
            .fold(Spectrum::fill(1.0), |acc, t| acc.mul(t))
    }

    pub fn pdf(&self) -> f64 {
        self.vertices
            .iter()
            .map(|v| v.forward_pdf.unwrap_or(1.0))
            .fold(1.0, |a, b| a * b)
    }

    pub fn weight(&self) -> f64 {
        let mut product = 1.0;
        let mut sum = 0.0;

        for vertex in self.vertices[0..self.technique.camera].iter().rev() {
            if let Some(w) = vertex.camera_path_weight() {
                if w != 0.0 {
                    product = product * w;
                    sum = sum + product;
                }
            }
        }

        if self.technique.light >= 2 {
            for vertex in self.vertices[self.technique.camera..].iter() {
                if let Some(w) = vertex.light_path_weight() {
                    if w != 0.0 {
                        product = product * w;
                        sum = sum + product;
                    }
                }
            }
        }

        1.0 / (1.0 + sum)
    }
}

#[cfg(test)]
mod tests {
    use super::{Contribution, PathType, Technique};
    use crate::{image::PixelCoordinates, sampler::test::MockSampler, spectrum::RgbSpectrum};

    #[test]
    fn test_technique_sample() {
        let mut sampler = MockSampler::new();

        sampler.add(0.0);
        let technique = Technique::sample(2, &mut sampler);
        assert_eq!(technique.camera, 0);
        assert_eq!(technique.light, 2);

        sampler.add(0.5);
        let technique = Technique::sample(2, &mut sampler);
        assert_eq!(technique.camera, 1);
        assert_eq!(technique.light, 1);

        sampler.add(0.99);
        let technique = Technique::sample(2, &mut sampler);
        assert_eq!(technique.camera, 2);
        assert_eq!(technique.light, 0);
    }

    #[test]
    fn test_technique_path_type() {
        let technique = Technique::new(2, 2);
        assert_eq!(technique.path_type(0), PathType::Camera);
        assert_eq!(technique.path_type(1), PathType::Camera);
        assert_eq!(technique.path_type(2), PathType::Light);
        assert_eq!(technique.path_type(3), PathType::Light);
    }

    #[test]
    fn test_contribution_acceptance() {
        let spectrum1 = RgbSpectrum::fill(0.1);
        let current = Contribution {
            scalar: spectrum1.luminance(),
            spectrum: spectrum1,
            pixel_coordinates: PixelCoordinates { x: 100, y: 100 },
        };

        let spectrum2 = RgbSpectrum::fill(0.05);
        let proposed = Contribution {
            scalar: spectrum2.luminance(),
            spectrum: spectrum2,
            pixel_coordinates: PixelCoordinates { x: 100, y: 100 },
        };

        let a = Contribution::acceptance(current, proposed);
        assert_eq!(a, 0.5);
    }
}
