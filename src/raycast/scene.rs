use std::convert::{TryFrom, TryInto};
use serde_json::{from_value, Value as SerdeValue, Error as SerdeError};

use crate::utils;
use crate::raycast::{
    general::{color::Color, Light, Intersect, Intersection},
    vector::Vector3,
    ray::Ray,
    objects::{TransformableObject3D, Object3D},
};

/// With this the scene description can specify the *object* and opt-in to have
/// a *transform* on each object (ie. the description does not _require_ a
/// transform field for any object)
#[derive(serde::Deserialize)]
struct TransformableObject3DRecord {
    #[serde(default)] // Uses Option<T>::default() if not present
    transform: Option<String>,
    object: Object3DRecord,
}
/// With this, `TransformableObject3DRecord` can specify objects either based on
/// string names pointing to predefined objects or describing them then and
/// there directly
#[derive(serde::Deserialize)]
enum Object3DRecord {
    Named(String),
    Raw(Object3D),
}

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

        // TODO The scene is described in JSON with different intersectable
        // objects named by the user. Here those names are turned into indices
        // into the said objects ie. the actual objects are allocated once and
        // in rendering used multiple times with different transformations

        let lights: Vec<Light> = from_value(json["lights"].take())?;

        let objects = {
            let named: std::collections::HashMap<String, Object3D> =
                from_value(json["named"].take())?;

            from_value::<Vec<TransformableObject3DRecord>>(
                    json["objects"].take()
                )?
                .iter()
                .map(|TransformableObject3DRecord{ transform, object }| {
                    // Parse transform matrix from string
                    let transform = transform.as_ref().map(|s|
                        s[..].try_into().expect("Bad transform string")
                    );
                    // Either choose the Raw object or clone if Named
                    // TODO implement reference counted version for named here
                    // (now, calling to_owned clones)
                    let object = match object {
                        Object3DRecord::Raw(o) => o.to_owned(),
                        Object3DRecord::Named(s) => {
                            named.get(s)
                                .expect(format!(
                                        "The named object '{}' is not found \
                                        in the 'objects' field", s
                                    ).as_str()
                                )
                                .to_owned()
                        },
                    };
                    TransformableObject3D::new(transform, object)
                })
                .collect()
        };

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
