use std::collections::VecDeque;

use crate::{
    geometry::Geometry,
    image::PixelCoordinates,
    interaction::{CameraInteraction, Interaction, LightInteraction, ObjectInteraction},
    ray::Ray,
    sampler::{MmltSampler, Sampler},
    scene::Scene,
    spectrum::Spectrum,
    util,
    vector::Vector,
};

pub struct Path<'a> {
    vertices: Vec<Vertex<'a>>,
    technique: Technique,
    pixel_coordinates: PixelCoordinates,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PathType {
    Camera,
    Light,
}

pub struct CameraVertex<'a> {
    interaction: CameraInteraction<'a>,
    wi: Vector,
    direction_to_area: f64,
    geometry_term: f64,
}

pub struct LightVertex<'a> {
    interaction: LightInteraction<'a>,
    wo: Vector,
    direction_to_area: f64,
}

pub struct ObjectVertex<'a> {
    interaction: ObjectInteraction<'a>,
    wo: Vector,
    wi: Vector,
    direction_to_area: f64,
    geometry_term: f64,
}

pub enum Vertex<'a> {
    Camera(CameraVertex<'a>),
    Light(LightVertex<'a>),
    Object(ObjectVertex<'a>),
}

impl<'a> Vertex<'a> {
    fn throughput(&self) -> Spectrum {
        match self {
            Vertex::Camera(v) => {
                v.interaction
                    .camera
                    .importance(v.interaction.geometry.point, v.wi)
                    * v.geometry_term
            }
            Vertex::Light(v) => v.interaction.light.radiance(
                v.interaction.geometry.point,
                v.interaction.geometry.normal,
                v.wo,
            ),
            Vertex::Object(v) => v.interaction.reflectance(v.wo, v.wi) * v.geometry_term,
        }
    }

    fn probability(&self, path_type: PathType) -> Option<f64> {
        let p = match self {
            Vertex::Camera(v) => {
                v.interaction
                    .camera
                    .probability(v.interaction.geometry.point, v.wi)?
                    * v.direction_to_area
            }
            Vertex::Light(v) => {
                v.interaction.light.probability(
                    v.interaction.geometry.point,
                    v.interaction.geometry.normal,
                    v.wo,
                )? * v.direction_to_area
            }
            Vertex::Object(v) => match path_type {
                PathType::Camera => v.interaction.probability(v.wo, v.wi)? * v.direction_to_area,
                PathType::Light => v.interaction.probability(v.wi, v.wo)? * v.direction_to_area,
            },
        };
        Some(p)
    }

    fn weight(&self, direction: Direction) -> Option<f64> {
        match self {
            Vertex::Camera(_) => Some(1.0),
            Vertex::Light(_) => Some(1.0),
            Vertex::Object(_) => {
                let rev = self.probability(PathType::Light)?;
                let fwd = self.probability(PathType::Camera)?;
                match direction {
                    Direction::Forward => Some(rev / fwd),
                    Direction::Reverse => Some(fwd / rev),
                }
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

enum Direction {
    Forward,
    Reverse,
}

#[derive(Copy, Clone)]
pub struct Contribution {
    pub scalar: f64,
    pub spectrum: Spectrum,
    pub pixel_coordinates: PixelCoordinates,
}

impl Contribution {
    pub fn ratio(&self, current_contribution: Contribution) -> f64 {
        if current_contribution.scalar > 0.0 {
            f64::max(
                f64::min(1.0, self.scalar / current_contribution.scalar),
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

impl<'a> Path<'a> {
    pub fn sampler() -> MmltSampler {
        MmltSampler::new(STREAM_COUNT)
    }

    pub fn generate(
        scene: &'a Scene,
        sampler: &mut impl Sampler,
        path_length: usize,
    ) -> Option<Path<'a>> {
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
        scene: &'a Scene,
        sampler: &mut impl Sampler,
        technique: Technique,
    ) -> Option<Path<'a>> {
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
        let mut interactions: VecDeque<Interaction<'a>> = VecDeque::new();
        interactions.push_back(camera_interaction);
        interactions.push_back(light_interaction);
        Path::connect(&mut interactions, technique)
    }

    fn connect_full_light_path(
        scene: &'a Scene,
        sampler: &mut impl Sampler,
        technique: Technique,
    ) -> Option<Path<'a>> {
        sampler.start_stream(LIGHT_STREAM);
        let light = scene.sample_light(sampler);
        let light_interaction = light.sample_interaction(sampler);
        let mut interactions = Path::trace(
            scene,
            sampler,
            light_interaction,
            technique.light,
            Direction::Reverse,
        )?;
        interactions.front().filter(|i| i.is_camera())?;
        Path::connect(&mut interactions, technique)
    }

    fn connect_full_camera_path(
        scene: &'a Scene,
        sampler: &mut impl Sampler,
        technique: Technique,
    ) -> Option<Path<'a>> {
        sampler.start_stream(CAMERA_STREAM);
        let camera_interaction = scene.camera.sample_interaction(sampler);
        let mut interactions = Path::trace(
            scene,
            sampler,
            camera_interaction,
            technique.camera,
            Direction::Forward,
        )?;
        interactions.back().filter(|i| i.is_light())?;
        Path::connect(&mut interactions, technique)
    }

    fn connect_camera_to_light_subpath(
        scene: &'a Scene,
        sampler: &mut impl Sampler,
        technique: Technique,
    ) -> Option<Path<'a>> {
        sampler.start_stream(LIGHT_STREAM);
        let light = scene.sample_light(sampler);
        let light_interaction = light.sample_interaction(sampler);
        let mut interactions = Path::trace(
            scene,
            sampler,
            light_interaction,
            technique.light,
            Direction::Reverse,
        )?;
        let last = interactions.front().filter(|i| i.is_object())?;
        sampler.start_stream(CAMERA_STREAM);
        let sampled_camera_interaction = scene.camera.sample_interaction(sampler);
        let ray = Ray::new(
            last.geometry().point,
            sampled_camera_interaction.geometry().point - last.geometry().point,
        );
        let camera_interaction = scene.intersect(ray).filter(|i| i.is_camera())?;
        interactions.push_back(camera_interaction);
        Path::connect(&mut interactions, technique)
    }

    fn connect_camera_subpath_to_light(
        scene: &'a Scene,
        sampler: &mut impl Sampler,
        technique: Technique,
    ) -> Option<Path<'a>> {
        sampler.start_stream(CAMERA_STREAM);
        let camera_interaction = scene.camera.sample_interaction(sampler);
        let mut interactions = Path::trace(
            scene,
            sampler,
            camera_interaction,
            technique.camera,
            Direction::Forward,
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
        scene: &'a Scene,
        sampler: &mut impl Sampler,
        technique: Technique,
    ) -> Option<Path<'a>> {
        sampler.start_stream(CAMERA_STREAM);
        let camera_interaction = scene.camera.sample_interaction(sampler);
        let camera_interactions = Path::trace(
            scene,
            sampler,
            camera_interaction,
            technique.camera,
            Direction::Forward,
        )?;
        sampler.start_stream(LIGHT_STREAM);
        let light = scene.sample_light(sampler);
        let light_interaction = light.sample_interaction(sampler);
        let light_interactions = Path::trace(
            scene,
            sampler,
            light_interaction,
            technique.light,
            Direction::Reverse,
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
        direction: Direction,
    ) -> Option<VecDeque<Interaction<'a>>> {
        let mut stack: VecDeque<Interaction<'a>> = VecDeque::new();
        let mut ray = interaction.initial_ray()?;
        match direction {
            Direction::Forward => stack.push_back(interaction),
            Direction::Reverse => stack.push_front(interaction),
        };
        for _ in 0..length {
            let interaction = scene.intersect(ray)?;
            ray = interaction.generate_ray(sampler)?;
            match direction {
                Direction::Forward => stack.push_back(interaction),
                Direction::Reverse => stack.push_front(interaction),
            };
        }
        Some(stack)
    }

    fn connect(
        interactions: &mut VecDeque<Interaction<'a>>,
        technique: Technique,
    ) -> Option<Path<'a>> {
        let mut vertices: Vec<Vertex<'a>> = Vec::new();

        let mut previous_geometry: Option<Geometry> = None;

        let mut i: usize = 0;

        let mut pixel_coordinates: Option<PixelCoordinates> = None;

        while interactions.len() > 0 {
            let interaction = interactions.pop_front()?;
            let current_geometry = interaction.geometry();
            let next_geometry = interactions.get(0).map(Interaction::geometry);
            match interaction {
                Interaction::Camera(camera_interaction) => {
                    pixel_coordinates.replace(camera_interaction.pixel_coordinates);
                    match technique.path_type(i) {
                        PathType::Camera => {
                            let direction_to_area = util::direction_to_area(
                                camera_interaction.geometry.direction,
                                next_geometry?.normal,
                            );
                            let geometry_term = util::geometry_term(
                                camera_interaction.geometry.direction,
                                camera_interaction.geometry.normal,
                                next_geometry?.normal,
                            );
                            let wi = camera_interaction.geometry.direction;
                            let camera_vertex = CameraVertex {
                                interaction: camera_interaction,
                                wi,
                                direction_to_area,
                                geometry_term,
                            };

                            vertices.push(Vertex::Camera(camera_vertex));
                        }
                        PathType::Light => {
                            let wi = camera_interaction.geometry.direction;
                            let camera_vertex = CameraVertex {
                                interaction: camera_interaction,
                                wi,
                                direction_to_area: 1.0,
                                geometry_term: 1.0,
                            };

                            vertices.push(Vertex::Camera(camera_vertex));
                        }
                    }
                }
                Interaction::Light(light_interaction) => match technique.path_type(i) {
                    PathType::Camera => {
                        let wo = light_interaction.geometry.direction * -1.0;
                        let light_vertex = LightVertex {
                            interaction: light_interaction,
                            wo,
                            direction_to_area: 1.0,
                        };

                        vertices.push(Vertex::Light(light_vertex));
                    }
                    PathType::Light => {
                        let direction_to_area = util::direction_to_area(
                            light_interaction.geometry.direction,
                            previous_geometry?.normal,
                        );
                        let wo = light_interaction.geometry.direction;
                        let light_vertex = LightVertex {
                            interaction: light_interaction,
                            wo,
                            direction_to_area,
                        };

                        vertices.push(Vertex::Light(light_vertex));
                    }
                },
                Interaction::Object(object_interaction) => match technique.path_type(i) {
                    PathType::Camera => {
                        let wi = next_geometry?.point - object_interaction.geometry.point;

                        let object_vertex = ObjectVertex {
                            interaction: object_interaction,
                            wo: current_geometry.direction * -1.0,
                            wi,
                            direction_to_area: util::direction_to_area(wi, next_geometry?.normal),
                            geometry_term: util::geometry_term(
                                wi,
                                current_geometry.normal,
                                next_geometry?.normal,
                            ),
                        };

                        vertices.push(Vertex::Object(object_vertex));
                    }
                    PathType::Light => {
                        let wo = previous_geometry?.point - object_interaction.geometry.point;
                        let object_vertex = ObjectVertex {
                            interaction: object_interaction,
                            wo,
                            wi: current_geometry.direction * -1.0,
                            direction_to_area: util::direction_to_area(
                                wo,
                                previous_geometry?.normal,
                            ),
                            geometry_term: util::geometry_term(
                                wo,
                                current_geometry.normal,
                                previous_geometry?.normal,
                            ),
                        };
                        vertices.push(Vertex::Object(object_vertex));
                    }
                },
            }
            previous_geometry.replace(current_geometry);
            i = i + 1;
        }

        let path = Path {
            vertices,
            technique,
            pixel_coordinates: pixel_coordinates?,
        };

        Some(path)
    }

    pub fn contribution(&self) -> Contribution {
        let c = self.throughput() * self.weight() / self.probability();
        Contribution {
            scalar: c.luminance(),
            spectrum: c,
            pixel_coordinates: self.pixel_coordinates,
        }
    }

    pub fn throughput(&self) -> Spectrum {
        self.vertices
            .iter()
            .map(|v| v.throughput())
            .fold(Spectrum::fill(1.0), |acc, t| acc.mul(t))
    }

    pub fn probability(&self) -> f64 {
        let camera_subpath = &self.vertices[0..self.technique.camera];
        let p1 = camera_subpath
            .iter()
            .map(|v| v.probability(PathType::Camera).unwrap_or(1.0))
            .fold(1.0, |acc, p| acc * p);
        let light_subpath = &self.vertices[self.technique.light..];
        let p2 = light_subpath
            .iter()
            .rev()
            .map(|v| v.probability(PathType::Light).unwrap_or(1.0))
            .fold(1.0, |acc, p| acc * p);
        p1 * p2
    }

    pub fn weight(&self) -> f64 {
        let camera_subpath = &self.vertices[0..self.technique.camera];
        let light_subpath = &self.vertices[self.technique.light..];
        let fold = |(product, sum): (f64, f64), result: Option<f64>| -> (f64, f64) {
            match result {
                Some(weight) => {
                    let p = product * weight;
                    (p, sum + p)
                }
                None => (product, sum),
            }
        };
        let (_, sum1) = camera_subpath
            .iter()
            .map(|v| v.weight(Direction::Forward))
            .fold((1.0, 0.0), fold);
        let (_, sum2) = light_subpath
            .iter()
            .rev()
            .map(|v| v.weight(Direction::Reverse))
            .fold((1.0, 0.0), fold);
        1.0 / (1.0 + sum1 + sum2)
    }
}

#[cfg(test)]
mod tests {
    use super::Technique;
    use crate::{path::PathType, sampler::test::MockSampler};

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
}
