use std::collections::VecDeque;

use crate::{
    bsdf::EvaluationContext,
    geometry::Geometry,
    interaction::Interaction,
    ray::Ray,
    sampler::{MmltSampler, Sampler},
    scene::Scene,
    spectrum::Spectrum,
    types::PathType,
    util,
    vector::Point2,
};

#[derive(Debug)]
pub struct Path {
    vertices: Vec<Vertex>,
    technique: Technique,
    pixel_coordinates: Point2,
}

#[derive(Debug)]
pub struct Vertex {
    throughput: Spectrum,
    forward_pdf: Option<f64>,
    reverse_pdf: Option<f64>,
}

impl Vertex {
    fn weight(&self) -> Option<f64> {
        if self.forward_pdf.is_none() && self.reverse_pdf.is_none() {
            return None;
        }
        let fwd = self.forward_pdf.unwrap_or(1.0);
        let rev = self.reverse_pdf.unwrap_or(1.0);
        if fwd == 0.0 {
            return None;
        }
        Some(rev / fwd)
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
        let r = sampler.sample(1.0..end);
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
    pub pixel_coordinates: Point2,
}

impl Contribution {
    pub fn empty() -> Contribution {
        Contribution {
            scalar: 0.0,
            spectrum: Spectrum::black(),
            pixel_coordinates: Point2::new(0.0, 0.0),
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
        let mut pixel_coordinates: Option<Point2> = None;
        let mut area_pdf: Option<f64> = None;
        let mut previous_geometry: Option<Geometry> = None;
        let mut previous_object_sampling_pdf: Option<f64> = None;
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
                    let combine = |area: Option<f64>, sampling: Option<f64>| {
                        if area.is_some() {
                            area.map(|a| a * sampling.unwrap_or(1.0))
                        } else {
                            sampling
                        }
                    };
                    let point = object_interaction.geometry.point;
                    let normal = object_interaction.geometry.normal;
                    let next_normal = next_geometry?.normal;
                    let wo = previous_geometry?.point - point;
                    let wi = next_geometry?.point - point;
                    let geometry_term = util::geometry_term(wi, normal, next_normal);
                    let context = EvaluationContext {
                        geometry_term,
                        path_type: technique.path_type(index),
                    };
                    let reflectance = object_interaction.reflectance(wo, wi, context);
                    let throughput = reflectance * geometry_term;
                    let current_object_sampling_pdf =
                        object_interaction.sampling_pdf(wo, wi, technique.path_type(index));
                    let vertex = match technique.path_type(index) {
                        PathType::Camera => Vertex {
                            throughput,
                            forward_pdf: combine(area_pdf, previous_object_sampling_pdf),
                            reverse_pdf: None,
                        },
                        PathType::Light => Vertex {
                            throughput,
                            forward_pdf: None,
                            reverse_pdf: combine(area_pdf, previous_object_sampling_pdf),
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
                            previous_vertex.reverse_pdf =
                                combine(previous_area_pdf, current_object_sampling_pdf);
                        }
                        PathType::Light => {
                            previous_vertex.forward_pdf =
                                combine(previous_area_pdf, current_object_sampling_pdf);
                        }
                    }
                    let next_normal = next_geometry?.normal;
                    let next_directional_pdf = object_interaction.pdf(wo, wi, PathType::Camera);
                    let next_direction_to_area = util::direction_to_area(wi, next_normal);
                    area_pdf = next_directional_pdf.map(|p| p * next_direction_to_area);
                    previous_object_sampling_pdf = current_object_sampling_pdf;
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
            if let Some(w) = vertex.weight() {
                product = product * w;
                sum = sum + product;
            }
        }

        product = 1.0;

        if self.technique.light >= 1 {
            for vertex in self.vertices[self.technique.camera..].iter() {
                if let Some(w) = vertex.weight() {
                    product = product * w;
                    sum = sum + product;
                }
            }
        }

        1.0 / (1.0 + sum)
    }
}

#[cfg(test)]
mod tests {
    use super::{Contribution, PathType, Technique};
    use crate::{sampler::test::MockSampler, spectrum::RgbSpectrum, vector::Point2};

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
            pixel_coordinates: Point2::new(100.0, 100.0),
        };

        let spectrum2 = RgbSpectrum::fill(0.05);
        let proposed = Contribution {
            scalar: spectrum2.luminance(),
            spectrum: spectrum2,
            pixel_coordinates: Point2::new(100.0, 100.0),
        };

        let a = Contribution::acceptance(current, proposed);
        assert_eq!(a, 0.5);
    }

    /*
    #[test]
    fn test_gpu() {
        let scene =
            Scene::load(String::from("/Users/david/Desktop/mmlt/scenes/scene-2.yml")).unwrap();
        let technique = Technique::new(3, 1);
        let mut interactions = VecDeque::new();
        let camera_interaction = CameraInteraction {
            camera: scene.camera.as_ref(),
            geometry: Geometry {
                point: Point3::new(50.0, 40.79999923706055, 220.0),
                normal: Vector3::new(0.0, 0.0, -1.0),
                direction: Vector3::new(
                    -0.36581405997276306,
                    -0.09223464131355286,
                    -0.9261062741279602,
                ),
            },
            pixel_coordinates: Vector2::new(0.0, 0.0),
        };
        interactions.push_front(Interaction::Camera(camera_interaction));

        let object_interaction_1 = ObjectInteraction {
            object: scene.objects[0].as_ref(),
            geometry: Geometry {
                point: Point3::new(1.0176239013671875, 28.449811935424805, 95.99468994140625),
                normal: Vector3::new(
                    -0.9999982118606567,
                    -0.0012350187171250582,
                    0.0014394691679626703,
                ),
                direction: Vector3::new(
                    -0.36581405997276306,
                    -0.09223464131355286,
                    -0.9261062741279602,
                ),
            },
            bsdf: OnceCell::new(),
        };
        interactions.push_back(Interaction::Object(object_interaction_1));

        let object_interaction_2 = ObjectInteraction {
            object: scene.objects[6].as_ref(),
            geometry: Geometry {
                point: Point3::new(30.91585922241211, 17.53809928894043, 55.45295715332031),
                normal: Vector3::new(
                    -0.9542068839073181,
                    -0.12309502065181732,
                    0.27264782786369324,
                ),
                direction: Vector3::new(
                    0.5800725221633911,
                    -0.2117042988538742,
                    -0.786573052406311,
                ),
            },
            bsdf: OnceCell::new(),
        };
        interactions.push_back(Interaction::Object(object_interaction_2));

        let light_interaction = LightInteraction {
            light: scene.lights[0].as_ref(),
            geometry: Geometry {
                point: Point3::new(8.662123680114746, 66.57731628417969, 46.85707473754883),
                normal: Vector3::new(
                    -0.2229793518781662,
                    -0.5704475045204163,
                    -0.7904869914054871,
                ),
                direction: Vector3::new(
                    -0.6243391036987305,
                    0.31962957978248596,
                    -0.712767481803894,
                ),
            },
        };
        interactions.push_back(Interaction::Light(light_interaction));

        let path = Path::connect(&mut interactions, technique).unwrap();

        println!("throughput = {:?}", path.throughput());
        println!("pdf = {:?}", path.pdf());
        println!("MIS weight = {}", path.weight());
        println!("beta = {:?}", path.throughput() / path.pdf());
        println!("contribution = {:?}", path.contribution());
    }
    */
}
