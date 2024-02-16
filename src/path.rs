use crate::{
    camera::Camera,
    intersection::{Intersection, Orientation},
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
            Vertex::Camera(v) => v.camera.probability(v.point, v.wi) * v.direction_to_area,
            Vertex::Light(v) => v.light.probability(v.wo) * v.direction_to_area,
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

pub enum ConnectionType {
    Trivial,
    DirectLight,
    DirectCamera,
    LightTrace,
    CameraTrace,
    Interior,
}

impl Technique {
    pub fn sample(path_length: usize, sampler: &mut impl Sampler) -> Technique {
        let r = sampler.sample(0.0..path_length as f64);
        let camera = (r * path_length as f64) as usize;
        let light = path_length - camera;
        Technique { camera, light }
    }

    pub fn connection_type(&self) -> ConnectionType {
        if self.camera == 0 {
            ConnectionType::DirectCamera
        } else if self.camera == 1 {
            if self.light == 1 {
                ConnectionType::Trivial
            } else {
                ConnectionType::CameraTrace
            }
        } else {
            if self.light == 0 {
                ConnectionType::DirectLight
            } else if self.light == 1 {
                ConnectionType::LightTrace
            } else {
                ConnectionType::Interior
            }
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

        match technique.connection_type() {
            ConnectionType::Trivial => Path::trivial(scene, sampler),
            ConnectionType::DirectCamera => Path::direct_camera(scene, sampler, technique),
            ConnectionType::DirectLight => Path::direct_light(scene, sampler, technique),
            ConnectionType::CameraTrace => Path::camera_trace(scene, sampler, technique),
            ConnectionType::LightTrace => Path::light_trace(scene, sampler, technique),
            ConnectionType::Interior => Path::interior(scene, sampler, technique),
        }
    }

    pub fn trivial(scene: &'a Scene, sampler: &mut impl Sampler) -> Option<Path<'a>> {
        sampler.start_stream(CAMERA_STREAM);
        let camera_intersection = scene.camera.sample_intersection(sampler);
        sampler.start_stream(LIGHT_STREAM);
        let light = scene.sample_light(sampler);
        let light_intersection = light.sample_intersection(sampler);
        let intersections = vec![camera_intersection, light_intersection];
        Path::compute(&intersections)
    }

    // TODO: sometimes we sample a point, sometimes a ray; this might require 1 or 2 random numbers; ensure consistency somehow
    pub fn direct_camera(
        scene: &'a Scene,
        sampler: &mut impl Sampler,
        technique: Technique,
    ) -> Option<Path<'a>> {
        sampler.start_stream(LIGHT_STREAM);
        let light = scene.sample_light(sampler);
        let light_intersection = light.sample_intersection(sampler);
        let intersections = Path::trace(scene, sampler, light_intersection, technique.light)?;
        intersections.last().filter(|i| i.is_camera())?;
        Path::compute(&intersections)
    }

    pub fn direct_light(
        scene: &'a Scene,
        sampler: &mut impl Sampler,
        technique: Technique,
    ) -> Option<Path<'a>> {
        sampler.start_stream(CAMERA_STREAM);
        let camera_intersection = scene.camera.sample_intersection(sampler);
        let intersections = Path::trace(scene, sampler, camera_intersection, technique.camera)?;
        intersections.last().filter(|i| i.is_light())?;
        Path::compute(&intersections)
    }

    pub fn camera_trace(
        scene: &'a Scene,
        sampler: &mut impl Sampler,
        technique: Technique,
    ) -> Option<Path<'a>> {
        sampler.start_stream(LIGHT_STREAM);
        let light = scene.sample_light(sampler);
        let light_intersection = light.sample_intersection(sampler);
        let mut intersections = Path::trace(scene, sampler, light_intersection, technique.light)?;
        let last = intersections.last().filter(|i| i.is_object())?;
        sampler.start_stream(CAMERA_STREAM);
        let camera_intersection = scene.camera.sample_intersection(sampler);
        let ray = Ray::new(last.point(), camera_intersection.point() - last.point());
        let intersection = scene.intersect(ray).filter(|i| i.is_camera())?;
        intersections.push(intersection);
        Path::compute(&intersections)
    }

    pub fn light_trace(
        scene: &'a Scene,
        sampler: &mut impl Sampler,
        technique: Technique,
    ) -> Option<Path<'a>> {
        sampler.start_stream(CAMERA_STREAM);
        let camera_intersection = scene.camera.sample_intersection(sampler);
        let mut intersections = Path::trace(scene, sampler, camera_intersection, technique.camera)?;
        let last = intersections.last().filter(|i| i.is_object())?;
        sampler.start_stream(LIGHT_STREAM);
        let light = scene.sample_light(sampler);
        let light_intersection = light.sample_intersection(sampler);
        let ray = Ray::new(last.point(), light_intersection.point() - last.point());
        let intersection = scene.intersect(ray).filter(|i| i.is_light())?;
        intersections.push(intersection);
        Path::compute(&intersections)
    }

    pub fn interior(
        scene: &'a Scene,
        sampler: &mut impl Sampler,
        technique: Technique,
    ) -> Option<Path<'a>> {
        sampler.start_stream(CAMERA_STREAM);
        let camera_intersection = scene.camera.sample_intersection(sampler);
        let camera_intersections =
            Path::trace(scene, sampler, camera_intersection, technique.camera)?;
        sampler.start_stream(LIGHT_STREAM);
        let light = scene.sample_light(sampler);
        let light_intersection = light.sample_intersection(sampler);
        let mut light_intersections =
            Path::trace(scene, sampler, light_intersection, technique.light)?;
        let camera_last = camera_intersections.last().filter(|i| i.is_object())?;
        let light_last = light_intersections.last().filter(|i| i.is_object())?;
        let ray = Ray::new(camera_last.point(), light_last.point() - light_last.point());
        let id = light_last.id();
        let intersection = scene.intersect(ray).filter(|i| i.id() == id)?;
        light_intersections.reverse();
        let mut intersections = camera_intersections;
        intersections.push(intersection);
        intersections.extend(light_intersections);
        Path::compute(&intersections)
    }

    fn trace(
        scene: &'a Scene,
        sampler: &mut impl Sampler,
        intersection: Intersection<'a>,
        length: usize,
    ) -> Option<Vec<Intersection<'a>>> {
        let mut stack: Vec<Intersection<'a>> = Vec::new();
        let mut ray = intersection.generate_ray(sampler)?;
        stack.push(intersection);
        for _ in 0..length {
            let intersection = scene.intersect(ray)?;
            ray = intersection.generate_ray(sampler)?;
            stack.push(intersection);
        }
        Some(stack)
    }

    fn compute(intersections: &Vec<Intersection<'a>>) -> Option<Path<'a>> {
        let mut vertices: Vec<Vertex<'a>> = Vec::new();

        let technique = Technique {
            camera: 0,
            light: 0,
        };

        for i in 0..intersections.len() {
            match &intersections[i] {
                Intersection::Camera(camera_intersection) => {
                    match camera_intersection.orientation {
                        Orientation::Camera => {
                            let next_intersection = &intersections[i + 1];

                            let camera_vertex = CameraVertex {
                                camera: camera_intersection.camera,
                                point: camera_intersection.point,
                                wi: camera_intersection.direction,
                                direction_to_area: util::direction_to_area(
                                    camera_intersection.direction,
                                    next_intersection.normal(),
                                ),
                                geometry_term: util::geometry_term(
                                    camera_intersection.direction,
                                    camera_intersection.normal,
                                    next_intersection.normal(),
                                ),
                            };

                            vertices.push(Vertex::Camera(camera_vertex));
                        }
                        Orientation::Light => {
                            let camera_vertex = CameraVertex {
                                camera: camera_intersection.camera,
                                point: camera_intersection.point,
                                wi: camera_intersection.direction,
                                direction_to_area: 1.0,
                                geometry_term: 1.0,
                            };

                            vertices.push(Vertex::Camera(camera_vertex));
                        }
                    }
                }
                Intersection::Light(light_intersection) => match light_intersection.orientation {
                    Orientation::Camera => {
                        let light_vertex = LightVertex {
                            light: light_intersection.light,
                            point: light_intersection.point,
                            wo: light_intersection.direction * -1.0,
                            normal: light_intersection.normal,
                            direction_to_area: 1.0,
                        };

                        vertices.push(Vertex::Light(light_vertex));
                    }
                    Orientation::Light => {
                        let previous_intersection = &intersections[i - 1];

                        let light_vertex = LightVertex {
                            light: light_intersection.light,
                            point: light_intersection.point,
                            wo: light_intersection.direction,
                            normal: light_intersection.normal,
                            direction_to_area: util::direction_to_area(
                                light_intersection.direction,
                                previous_intersection.normal(),
                            ),
                        };

                        vertices.push(Vertex::Light(light_vertex));
                    }
                },
                Intersection::Object(object_intersection) => {
                    match object_intersection.orientation {
                        Orientation::Camera => {
                            let next_intersection = &intersections[i + 1];

                            let wi = next_intersection.point() - object_intersection.point;

                            let object_vertex = ObjectVertex {
                                object: object_intersection.object,
                                normal: object_intersection.normal,
                                point: object_intersection.point,
                                wo: object_intersection.direction * -1.0,
                                wi,
                                direction_to_area: util::direction_to_area(
                                    wi,
                                    next_intersection.normal(),
                                ),
                                geometry_term: util::geometry_term(
                                    wi,
                                    object_intersection.normal,
                                    next_intersection.normal(),
                                ),
                            };

                            vertices.push(Vertex::Object(object_vertex));
                        }
                        Orientation::Light => {
                            let previous_intersection = &intersections[i - 1];
                            let wo = previous_intersection.point() - object_intersection.point;
                            let object_vertex = ObjectVertex {
                                object: object_intersection.object,
                                normal: object_intersection.normal,
                                point: object_intersection.point,
                                wo,
                                wi: object_intersection.direction * -1.0,
                                direction_to_area: util::direction_to_area(
                                    wo,
                                    previous_intersection.normal(),
                                ),
                                geometry_term: util::geometry_term(
                                    wo,
                                    object_intersection.normal,
                                    previous_intersection.normal(),
                                ),
                            };
                            vertices.push(Vertex::Object(object_vertex));
                        }
                    }
                }
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
