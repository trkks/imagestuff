use std::convert::TryFrom;
use serde_json::{from_value, Value as SerdeValue, Error as SerdeError};

use crate::utils;
use crate::raycast::{
    general::{color::Color, Light, Intersect, Intersection},
    group::Group,
    ray::Ray,
    objects::Object3D,
    vector::{Vector3, Vector4},
};

/// A collection of transformable object-groups
pub struct Scene {
    pub ambient_color: Color,
    pub fov: f32,
    lights: Vec<Group<Light>>,
    objects: Vec<Group<Object3D>>,
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

                for Group { transformation, members } in &self.lights {
                    for light in members {
                        let (light_distance, towards_light) = {
                            // The light's position must be transformed first
                            let v4 = &Vector4::from_v3(light.position, 1.0);
                            let v = (transformation * v4).xyz() - intr.point;
                            (v.length(), v.normalized())
                        };

                        // Shadows:
                        let shadow_ray = Ray::with_transform(
                            off_surface,
                            towards_light,
                            transformation
                        );

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

    fn try_from(json: &'a mut SerdeValue) -> Result<Self,SerdeError> {
        let ambient_color = from_value(json["ambient_color"].take())?;

        // NOTE `fov` is turned into radians from the degrees in JSON
        let fov = utils::degs_to_rads(from_value(json["fov"].take())?);

        // NOTE Bad groups cause panic
        let lights: Vec<Group<Light>> = 
            from_value::<Vec<SerdeValue>>(json["lights"].take())?
            .into_iter()
            .map(|x| Group::try_from(x).unwrap())
            .collect();

        let objects: Vec<Group<Object3D>> = 
            from_value::<Vec<SerdeValue>>(json["3d"].take())?
            .into_iter()
            .map(|x| Group::try_from(x).unwrap())
            .collect();

        Ok(Scene { ambient_color, fov, lights, objects })
    }
}

impl Intersect for Scene {
    fn intersect(&self, ray: &Ray, tmin: f32) -> Option<Intersection> {
        //TODO intersect lights? (simulate a lens as glass sphere over camera)
        self.objects.iter()
            .filter_map(|group| group.intersect(&ray, tmin))
            .reduce(|acc, x| if x.t < acc.t { x } else { acc })
    }
}
