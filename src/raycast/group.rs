use std::convert::TryFrom;
use serde_json::{from_value, Value as SerdeValue, Error as SerdeError};

use crate::raycast::{
    general::{
        Intersect,
        Intersection,
    },
    ray::Ray,
    vector::Vector4,
    matrix::SquareMatrix4,
};

/// A collection of objects transformed inside a scene as a single group. 
/// This is needed mainly for "code clarity" and in order to parse both the
/// transformation _and_ objects from some source (ie. a json file).
pub struct Group<T> {
    pub transformation: SquareMatrix4,
    pub members: Vec<T>,
}

impl<T: Intersect> Intersect for Group<T> {
    fn intersect(&self, ray: &Ray, tmin: f32) -> Option<Intersection> {
        let transformed_ray = Ray::with_transform(
            ray.origin,
            ray.direction,
            &self.transformation
        );
        self.members.iter()
            .filter_map(|obj| obj.intersect(&transformed_ray, tmin))
            // Select the intersection closest to ray
            .reduce(|acc, x| if x.t < acc.t { x } else { acc })
            // Transform the intersection normal to object space
            .map(|mut intr| {
                let normal_v4 = Vector4::from_v3(intr.normal.into(), 0.0);
                // TODO Is this transformation right? (see also ray.rs)
                intr.normal = (&self.transformation.transposed() * &normal_v4)
                    .xyz()
                    .normalized();

                intr
            })
    }
}

impl<T> TryFrom<SerdeValue> for Group<T>
where
    T: TryFrom<SerdeValue>,
    <T as TryFrom<SerdeValue>>::Error: std::fmt::Debug,
{
    type Error = SerdeError;

    fn try_from(mut json: SerdeValue) -> Result<Self, Self::Error> {
        let transformation = {
            let transform_str = from_value::<String>(json["transform"].take())
                // A transformation is optional in json-file
                .unwrap_or_default();
            SquareMatrix4::try_from(&transform_str[..])
                .unwrap()
                // NOTE Inversed here in advance, because always used so
                .inversed()
        };

        let mut members = Vec::with_capacity(128);
        for obj in from_value::<Vec<SerdeValue>>(json["objects"].take())? {
            let t = T::try_from(obj).unwrap();
            members.push(t);
        }

        Ok(Group { transformation, members } )
    }
}
