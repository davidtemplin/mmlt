use crate::{
    camera::Camera,
    interaction::{Interaction, Orientation},
    light::Light,
    object::Object,
    ray::Ray,
    sampler::Sampler,
    scene::Scene,
    spectrum::Spectrum,
    util,
    vector::{Point, Vector},
};

pub struct Path<'a> {
    vertices: Vec<Vertex<'a>>,
    technique: Technique,
    x: usize,
    y: usize,
}

pub enum PathType {
    Camera,
    Light,
}

pub struct CameraVertex<'a> {
    camera: &'a (dyn Camera + 'a),
    point: Point,
    wi: Vector,
    direction_to_area: f64,
    geometry_term: f64,
}

pub struct LightVertex<'a> {
    light: &'a (dyn Light + 'a),
    point: Point,
    wo: Vector,
    normal: Vector,
    direction_to_area: f64,
}

pub struct ObjectVertex<'a> {
    object: &'a (dyn Object + 'a),
    point: Point,
    normal: Vector,
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
            Vertex::Camera(v) => v.camera.importance(v.point, v.wi) * v.geometry_term,
            Vertex::Light(v) => v.light.radiance(v.wo, v.normal),
            Vertex::Object(v) => v.object.reflectance(v.wo, v.normal, v.wi) * v.geometry_term,
        }
    }

    fn probability(&self, path_type: PathType) -> f64 {
        match self {
            Vertex::Camera(v) => v.camera.probability(v.point, v.wi) * v.direction_to_area, // TODO: need to let camera determine PDF, store it; could take more than 1 sample
            Vertex::Light(v) => v.light.probability(v.wo) * v.direction_to_area, // TODO: need to include PDF of sampling light from scene
            Vertex::Object(v) => match path_type {
                PathType::Camera => {
                    v.object.probability(v.wo, v.normal, v.wi) * v.direction_to_area
                }
                PathType::Light => v.object.probability(v.wi, v.normal, v.wo) * v.direction_to_area,
            },
        }
    }

    fn weight(&self, direction: Direction) -> Option<f64> {
        // TODO: dirac distributions; make probability() return Option?
        match self {
            Vertex::Camera(_) => Some(1.0),
            Vertex::Light(_) => Some(1.0),
            Vertex::Object(_) => {
                let rev = self.probability(PathType::Light);
                let fwd = self.probability(PathType::Camera);
                match direction {
                    Direction::Forward => Some(rev / fwd),
                    Direction::Reverse => Some(fwd / rev),
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
pub struct Technique {
    camera: usize,
    light: usize,
}

impl Technique {
    pub fn sample(path_length: usize, sampler: &mut impl Sampler) -> Technique {
        let r = sampler.sample(0.0..path_length as f64);
        let camera = (r * path_length as f64) as usize;
        let light = path_length - camera;
        Technique { camera, light }
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
    pub x: usize,
    pub y: usize,
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

impl<'a> Path<'a> {
    pub fn generate(
        scene: &'a Scene,
        sampler: &mut impl Sampler,
        path_length: usize,
    ) -> Option<Path<'a>> {
        sampler.start_iteration();
        sampler.start_stream(TECHNIQUE_STREAM);

        let technique = Technique::sample(path_length, sampler);

        if technique.camera == 0 {
            Path::connect_full_light_path(scene, sampler, technique)
        } else if technique.camera == 1 {
            if technique.light == 1 {
                Path::connect_camera_to_light(scene, sampler)
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

    pub fn connect_camera_to_light(
        scene: &'a Scene,
        sampler: &mut impl Sampler,
    ) -> Option<Path<'a>> {
        sampler.start_stream(CAMERA_STREAM);
        let camera_interaction = scene.camera.sample_interaction(sampler);
        sampler.start_stream(LIGHT_STREAM);
        let light = scene.sample_light(sampler);
        let light_interaction = light.sample_interaction(sampler);
        let interactions = vec![camera_interaction, light_interaction];
        Path::compute(&interactions)
    }

    // TODO: sometimes we sample a point, sometimes a ray; this might require 1 or 2 random numbers; ensure consistency somehow
    pub fn connect_full_light_path(
        scene: &'a Scene,
        sampler: &mut impl Sampler,
        technique: Technique,
    ) -> Option<Path<'a>> {
        sampler.start_stream(LIGHT_STREAM);
        let light = scene.sample_light(sampler);
        let light_interaction = light.sample_interaction(sampler);
        let interactions = Path::trace(scene, sampler, light_interaction, technique.light)?;
        interactions.last().filter(|i| i.is_camera())?;
        Path::compute(&interactions)
    }

    pub fn connect_full_camera_path(
        scene: &'a Scene,
        sampler: &mut impl Sampler,
        technique: Technique,
    ) -> Option<Path<'a>> {
        sampler.start_stream(CAMERA_STREAM);
        let camera_interaction = scene.camera.sample_interaction(sampler);
        let interactions = Path::trace(scene, sampler, camera_interaction, technique.camera)?;
        interactions.last().filter(|i| i.is_light())?;
        Path::compute(&interactions)
    }

    pub fn connect_camera_to_light_subpath(
        scene: &'a Scene,
        sampler: &mut impl Sampler,
        technique: Technique,
    ) -> Option<Path<'a>> {
        sampler.start_stream(LIGHT_STREAM);
        let light = scene.sample_light(sampler);
        let light_interaction = light.sample_interaction(sampler);
        let mut interactions = Path::trace(scene, sampler, light_interaction, technique.light)?;
        let last = interactions.last().filter(|i| i.is_object())?;
        sampler.start_stream(CAMERA_STREAM);
        let camera_interaction = scene.camera.sample_interaction(sampler);
        let ray = Ray::new(last.point(), camera_interaction.point() - last.point());
        let interaction = scene.intersect(ray).filter(|i| i.is_camera())?;
        interactions.push(interaction);
        Path::compute(&interactions)
    }

    pub fn connect_camera_subpath_to_light(
        scene: &'a Scene,
        sampler: &mut impl Sampler,
        technique: Technique,
    ) -> Option<Path<'a>> {
        sampler.start_stream(CAMERA_STREAM);
        let camera_interaction = scene.camera.sample_interaction(sampler);
        let mut interactions = Path::trace(scene, sampler, camera_interaction, technique.camera)?;
        let last = interactions.last().filter(|i| i.is_object())?;
        sampler.start_stream(LIGHT_STREAM);
        let light = scene.sample_light(sampler);
        let light_interaction = light.sample_interaction(sampler);
        let ray = Ray::new(last.point(), light_interaction.point() - last.point());
        let interaction = scene.intersect(ray).filter(|i| i.is_light())?;
        interactions.push(interaction);
        Path::compute(&interactions)
    }

    pub fn connect_camera_subpath_to_light_subpath(
        scene: &'a Scene,
        sampler: &mut impl Sampler,
        technique: Technique,
    ) -> Option<Path<'a>> {
        sampler.start_stream(CAMERA_STREAM);
        let camera_interaction = scene.camera.sample_interaction(sampler);
        let camera_interactions =
            Path::trace(scene, sampler, camera_interaction, technique.camera)?;
        sampler.start_stream(LIGHT_STREAM);
        let light = scene.sample_light(sampler);
        let light_interaction = light.sample_interaction(sampler);
        let mut light_interactions =
            Path::trace(scene, sampler, light_interaction, technique.light)?;
        let camera_last = camera_interactions.last().filter(|i| i.is_object())?;
        let light_last = light_interactions.last().filter(|i| i.is_object())?;
        let ray = Ray::new(camera_last.point(), light_last.point() - light_last.point());
        let id = light_last.id();
        let interaction = scene.intersect(ray).filter(|i| i.id() == id)?;
        light_interactions.reverse();
        let mut interactions = camera_interactions;
        interactions.push(interaction);
        interactions.extend(light_interactions);
        Path::compute(&interactions)
    }

    fn trace(
        scene: &'a Scene,
        sampler: &mut impl Sampler,
        interaction: Interaction<'a>,
        length: usize,
    ) -> Option<Vec<Interaction<'a>>> {
        let mut stack: Vec<Interaction<'a>> = Vec::new();
        let mut ray = interaction.generate_ray(sampler)?;
        stack.push(interaction);
        for _ in 0..length {
            let interaction = scene.intersect(ray)?;
            ray = interaction.generate_ray(sampler)?;
            stack.push(interaction);
        }
        Some(stack)
    }

    fn compute(interactions: &Vec<Interaction<'a>>) -> Option<Path<'a>> {
        let mut vertices: Vec<Vertex<'a>> = Vec::new();

        let technique = Technique {
            camera: 0,
            light: 0,
        };

        for i in 0..interactions.len() {
            match &interactions[i] {
                Interaction::Camera(camera_interaction) => match camera_interaction.orientation {
                    Orientation::Camera => {
                        let next_interaction = &interactions[i + 1];

                        let camera_vertex = CameraVertex {
                            camera: camera_interaction.camera,
                            point: camera_interaction.point,
                            wi: camera_interaction.direction,
                            direction_to_area: util::direction_to_area(
                                camera_interaction.direction,
                                next_interaction.normal(),
                            ),
                            geometry_term: util::geometry_term(
                                camera_interaction.direction,
                                camera_interaction.normal,
                                next_interaction.normal(),
                            ),
                        };

                        vertices.push(Vertex::Camera(camera_vertex));
                    }
                    Orientation::Light => {
                        let camera_vertex = CameraVertex {
                            camera: camera_interaction.camera,
                            point: camera_interaction.point,
                            wi: camera_interaction.direction,
                            direction_to_area: 1.0,
                            geometry_term: 1.0,
                        };

                        vertices.push(Vertex::Camera(camera_vertex));
                    }
                },
                Interaction::Light(light_interaction) => match light_interaction.orientation {
                    Orientation::Camera => {
                        let light_vertex = LightVertex {
                            light: light_interaction.light,
                            point: light_interaction.point,
                            wo: light_interaction.direction * -1.0,
                            normal: light_interaction.normal,
                            direction_to_area: 1.0,
                        };

                        vertices.push(Vertex::Light(light_vertex));
                    }
                    Orientation::Light => {
                        let previous_interaction = &interactions[i - 1];

                        let light_vertex = LightVertex {
                            light: light_interaction.light,
                            point: light_interaction.point,
                            wo: light_interaction.direction,
                            normal: light_interaction.normal,
                            direction_to_area: util::direction_to_area(
                                light_interaction.direction,
                                previous_interaction.normal(),
                            ),
                        };

                        vertices.push(Vertex::Light(light_vertex));
                    }
                },
                Interaction::Object(object_interaction) => match object_interaction.orientation {
                    Orientation::Camera => {
                        let next_interaction = &interactions[i + 1];

                        let wi = next_interaction.point() - object_interaction.point;

                        let object_vertex = ObjectVertex {
                            object: object_interaction.object,
                            normal: object_interaction.normal,
                            point: object_interaction.point,
                            wo: object_interaction.direction * -1.0,
                            wi,
                            direction_to_area: util::direction_to_area(
                                wi,
                                next_interaction.normal(),
                            ),
                            geometry_term: util::geometry_term(
                                wi,
                                object_interaction.normal,
                                next_interaction.normal(),
                            ),
                        };

                        vertices.push(Vertex::Object(object_vertex));
                    }
                    Orientation::Light => {
                        let previous_interaction = &interactions[i - 1];
                        let wo = previous_interaction.point() - object_interaction.point;
                        let object_vertex = ObjectVertex {
                            object: object_interaction.object,
                            normal: object_interaction.normal,
                            point: object_interaction.point,
                            wo,
                            wi: object_interaction.direction * -1.0,
                            direction_to_area: util::direction_to_area(
                                wo,
                                previous_interaction.normal(),
                            ),
                            geometry_term: util::geometry_term(
                                wo,
                                object_interaction.normal,
                                previous_interaction.normal(),
                            ),
                        };
                        vertices.push(Vertex::Object(object_vertex));
                    }
                },
            }
        }

        // TODO: compute this!
        let path = Path {
            vertices,
            technique,
            x: 0,
            y: 0,
        };

        Some(path)
    }

    pub fn contribution(&self) -> Contribution {
        let c = self.throughput() * self.weight() / self.probability();
        Contribution {
            scalar: c.luminance(),
            spectrum: c,
            x: self.x,
            y: self.y,
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
            .map(|v| v.probability(PathType::Camera))
            .fold(1.0, |acc, p| acc * p);
        let light_subpath = &self.vertices[self.technique.light..];
        let p2 = light_subpath
            .iter()
            .rev()
            .map(|v| v.probability(PathType::Light))
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
            .fold((0.0, 1.0), fold);
        let (_, sum2) = light_subpath
            .iter()
            .rev()
            .map(|v| v.weight(Direction::Reverse))
            .fold((0.0, 1.0), fold);
        1.0 / (1.0 + sum1 + sum2)
    }
}
