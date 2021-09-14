use std::convert::{TryFrom, TryInto};
use std::collections;
use serde_json::{from_value, Value as SerdeValue, Error as SerdeError};

use crate::utils;
use crate::raycast::{
    general::{color::Color, Light, Intersect, Intersection},
    vector::Vector3,
    ray::Ray,
    objects::{self, TransformableObject3D, Object3D},
};


/// A collection of things used in rendering a scene
pub struct Scene {
    pub ambient_color: Color,
    pub fov: f32,
    lights: Vec<Light>,
    objects: Vec<TransformableObject3D>
}

impl Scene {
    /// Recursive function that traces the ray `n` times
    pub fn trace(&self, ray: &Ray, n: usize) -> Color {
        // Shade with ambient color each time
        let mut color = self.ambient_color;
        if n > 0 {
            // TODO is epsilon needed here?
            if let Some(intr) = self.intersect(&ray, f32::EPSILON) {
                // Nudge off of the surface so that ray does not re-collide
                // (see "shadow acne")
                // NOTE The "bias" (ie. normal * epsilon) seems hard to get
                // right
                let off_surface =
                    intr.point + (intr.normal * 0.0001);

                for light in &self.lights {
                    let (light_distance, towards_light) = {
                        let v = light.position - intr.point;
                        (v.length(), v.normalized())
                    };

                    // Shadows:
                    let shadow_ray = Ray {
                        origin: off_surface,
                        direction: towards_light,
                    };

                    // If shadow ray does not cast shadow, color the point
                    if self.intersect(&shadow_ray, f32::EPSILON).is_none() {
                        // Shading model from:
                        // http://www.cs.cornell.edu/courses/cs4620/2014fa/lectures/05rt-shading.pdf
                        let intensity = light.intensity / light_distance;
                        let bisector = {
                            let v: Vector3 = (-intr.incoming).into();
                            let w: Vector3 = towards_light.into();
                            (v + w).normalized()
                        };

                        let d = intr.normal.dot(&towards_light);
                        if d >= 0.0 {
                            let s = intr.normal.dot(&bisector);
                            color += &(
                                // Diffuse
                                  intr.material.color
                                * intensity
                                * d
                                // Specular
                                + light.color
                                * intensity
                                * f32::max(0.0, s)
                                   .powi(intr.material.shininess)
                            );
                        }
                    }
                }

                // Reflections: Add color seen by reflected ray to current ray
                let reflected_ray = Ray {
                    origin: off_surface,
                    direction: intr.incoming.reflect(&intr.normal)
                };

                // Recursive call:
                return color + self.trace(&reflected_ray, n - 1)
            }
        }
        // End recursion:
        color
    }
}

impl<'a> TryFrom<&'a mut SerdeValue> for Scene {
    type Error = SerdeError;

    /// # Panics:
    /// This panics if the json description is invalid
    fn try_from(json: &'a mut SerdeValue) -> Result<Self, SerdeError> {
        let ambient_color = from_value(json["ambient_color"].take())?;

        // NOTE `fov` is turned into radians from the degrees in JSON
        let fov = utils::degs_to_rads(from_value(json["fov"].take())?);

        // The scene is described in JSON with different intersectable objects
        // named by the user. Here those names are turned into indices into the
        // said objects ie. the actual objects are allocated once and in
        // rendering used multiple times with different transformations

        let lights: Vec<Light> = from_value(json["lights"].take())?;

        let mut named = collections::HashMap::new();
        if let SerdeValue::Object(map) = json["named"].take() {
            named.reserve(map.len());
            for (key, value) in map {
                let object = if value.is_object() {
                    Object3D::Single(
                        from_value::<objects::Primitive3D>(value)?
                    )
                } else if value.is_array() {
                    Object3D::Composite(
                        from_value::<Vec<objects::Primitive3D>>(value)?
                    )
                } else {
                    panic!("The key '{}' in 'named' does not match to an \
                            object or array", key)
                };

                named.insert(key.to_string(), object);
            }
        } else {
            panic!("The key 'named' does not match to an object")
        }

        let mut objects = Vec::new();
        if let SerdeValue::Array(vec) = json["objects"].take() {
            objects.reserve(vec.len());
            for (i, mut value) in vec.into_iter().enumerate() {
                // Parse transform matrix from string
                let transform = from_value::<Option<String>>(
                        value["transform"].take()
                    )?
                    .as_ref()
                    .map(|s| s[..].try_into()
                        .expect(
                            format!("Bad transform string on the {} item in \
                                    'objects'", i).as_str()
                        )
                    );

                // Either create the raw object or choose from named ones
                let object = {
                    let obj = value["object"].take();
                    if obj.is_object() {
                        Object3D::Single(
                            from_value::<objects::Primitive3D>(obj)?
                        )
                    } else if obj.is_array() {
                        Object3D::Composite(
                            from_value::<Vec<objects::Primitive3D>>(obj)?
                        )
                    } else if obj.is_string() {
                        // Pick the object from the map of named ones
                        // TODO implement reference counted version for named
                        // here (now calling clone)
                        let key = obj.as_str().unwrap();
                        named.get(key)
                            .expect(
                                format!("The key '{}' is not found in \
                                        'objects'", key).as_str()
                            )
                            .clone()
                    } else {
                        panic!("The {} item in 'objects' is not an object,
                                array or string", i)
                    }
                };

                objects.push(TransformableObject3D::new(transform, object));
            }
        } else {
            panic!("The key 'objects' does not match to an array")
        }

        Ok(Scene { ambient_color, fov, lights, objects })
    }
}

impl Intersect for Scene {
    fn intersect(&self, ray: &Ray, tmin: f32) -> Option<Intersection> {
        //TODO intersect lights? (simulate a lens as glass sphere over camera)
        self.objects.iter()
            .filter_map(|x| x.intersect(&ray, tmin))
            .reduce(|acc, x| if x.t < acc.t { x } else { acc })
    }
}
