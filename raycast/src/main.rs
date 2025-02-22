use raycast::{
    ray::Ray,
    vector::{Vector, Vector3},
};

pub fn main() {
    let aspect_ratio = 16.0 / 9.0;
    let width = 256;
    let height = (width as f64 / aspect_ratio) as u32;
    assert!(height > 0);

    // Camera:
    let focal_length = 1.0;
    let viewport_height = 2.0;
    let viewport_width = viewport_height * (width as f32 / height as f32);
    let camera_center = Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    // Vectors across viewport edges.
    let viewport_u = Vector3 {
        x: viewport_width,
        y: 0.0,
        z: 0.0,
    };
    let viewport_v = Vector3 {
        x: 0.0,
        y: -viewport_height,
        z: 0.0,
    };

    // Space between two adjacent pixels.
    let pixel_u = viewport_u / width as f32;
    let pixel_v = viewport_v / height as f32;

    // Location of the top-left pixel.
    let viewport_top_left = camera_center
        - Vector3 {
            x: 0.0,
            y: 0.0,
            z: focal_length,
        }
        - viewport_u / 2.0
        - viewport_v / 2.0;
    let first_pixel_position = viewport_top_left + 0.5 * (pixel_u + pixel_v);

    println!("P3");
    println!("{} {}", width, height);
    println!("255");
    for j in 0..height {
        for i in 0..width {
            let pixel_center = first_pixel_position + (i as f32) * pixel_u + (j as f32) * pixel_v;
            let ray_dir = pixel_center - camera_center;
            let r = Ray {
                origin: camera_center,
                direction: ray_dir.normalized(),
            };
            let color = raycast::shade(r);
            let [r, g, b] = color.into();
            println!("{} {} {}", r, g, b);
        }
    }
}
