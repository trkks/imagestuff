use crate::{
    ray::Ray,
    vector::{UnitVector3, Vector3},
};

#[derive(Debug)]
pub struct PerspectiveCamera {
    horizontal: UnitVector3,
    up: UnitVector3,
    direction: UnitVector3,
    position: Vector3,
    dist_to_image: f32,
    aspect_ratio: f32,
    image_width: usize,
    image_height: usize,
}
impl PerspectiveCamera {
    /// Constructs a "normalized"(?) camera, meaning it points along the
    /// negative z-axis and is placed at [0, 0, 1], a unit away from the origin
    /// (at which the image plane is assumed to be).
    ///
    /// A custom field of view `fov` is given in radians and the image plane's
    /// aspect ratio is calculated from `image_width` and `image_height`.
    pub fn with_view(fov: f32, image_width: usize, image_height: usize) -> Self {
        let horizontal = Vector3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        }
        .normalized();
        let up = Vector3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        }
        .normalized();
        let direction = Vector3 {
            x: 0.0,
            y: 0.0,
            z: -1.0,
        }
        .normalized();
        let position = Vector3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        };

        // NOTE This assumes that x and y on image plane will be in [-1, 1]
        let dist_to_image = 1.0 / f32::tan(fov / 2.0);

        let aspect_ratio = image_width as f32 / image_height as f32;

        PerspectiveCamera {
            horizontal,
            up,
            direction,
            position,
            dist_to_image,
            aspect_ratio,
            image_width,
            image_height,
        }
    }

    pub fn image_dimensions(&self) -> (usize, usize) {
        (self.image_width, self.image_height)
    }

    pub fn shoot_at(&self, x: f32, y: f32) -> Ray {
        // Generate ray from camera to the image plane
        let ray_direction = x * self.horizontal * self.aspect_ratio
            + y * self.up
            + self.dist_to_image * self.direction
            - self.position;

        Ray {
            origin: self.position,
            direction: ray_direction.normalized(),
        }
    }
}
