use std::sync;

use terminal_toys as tt;

use crate::{camera, color, scene};

const AA_ITERATION_COUNT: usize = 25;

pub struct Raycaster {
    pub scene: scene::Scene,
    pub camera: camera::PerspectiveCamera,
}

impl Raycaster {
    /// Render the scene into a one-dimensional array of RGB-bytes (i.e., three
    /// (3) bytes per pixel) using `thread_count` concurrent threads.
    pub fn render_rgb_flat(self, thread_count: usize, is_debug: bool) -> Vec<u8> {
        let (width, height) = self.camera.image_dimensions();
        let segment_height = height / thread_count;
        let mut img_threads = Vec::with_capacity(thread_count);

        println!("Rendering:");

        let camera = { sync::Arc::new(self.camera) };
        let scene = { sync::Arc::new(self.scene) };

        // Spawn the threads to render in
        for (i, mut progress_bar) in tt::ProgressBar::multiple(width * height, 25, thread_count)
            .into_iter()
            .enumerate()
        {
            let arc_camera = sync::Arc::clone(&camera);
            let arc_scene = sync::Arc::clone(&scene);

            // Every pixel in segment counts towards progress
            progress_bar.title(&format!("  Thread #{} progress", i + 1));

            let y_range = {
                // Iterate the coordinates in image segments
                let start = i * segment_height;
                let mut end = start + segment_height;
                // The last created thread takes the remaining rows as well
                if i == thread_count - 1 {
                    end += height % thread_count;
                }
                start..end
            };

            let mut img_vec = Vec::with_capacity(width * y_range.len());

            img_threads.push(std::thread::spawn(move || {
                for iy in y_range {
                    for ix in 0..width {
                        img_vec.extend_from_slice(&shade_pixel(
                            ix,
                            iy,
                            width,
                            height,
                            &arc_camera,
                            &arc_scene,
                            is_debug,
                        ));
                        progress_bar.lap().expect("Progress bar print failure");
                    }
                }
                // Return the rendered pixels in segment
                img_vec
            }));
        }

        // Wait for rendering threads to finish and combine the rendered segments
        // in order
        let mut img_combined = Vec::with_capacity(width * height);
        for t in img_threads {
            img_combined.append(&mut t.join().unwrap());
        }

        // Move command line cursor to bottom of progress bars
        print!("\x1b[{}B", thread_count);

        img_combined
    }
}

fn shade_pixel(
    ix: usize,
    iy: usize,
    width: usize,
    height: usize,
    camera: &sync::Arc<camera::PerspectiveCamera>,
    scene: &sync::Arc<scene::Scene>,
    debug: bool,
) -> [u8; 3] {
    let mut color = color::consts::BLACK;

    // Anti-aliasing: sample each pixel in some pattern and return average
    // TODO Separate AA into a general function; AA works on a pixel:
    // 1) get location and size of a pixel (input: rectangle)
    // 2) shoot rays into these bounds (output: coordinates)
    for _ in 0..AA_ITERATION_COUNT {
        let (tx, ty) = (rand::random::<f32>(), rand::random::<f32>());
        // Calculate image plane coordinates x,y so that they're in [-1, 1]
        let x: f32 = (ix as f32 + tx) / width as f32 * 2.0 - 1.0;
        // y is negated to transform from raster-space (ie. origin top left)
        // into screen-space (origin bottom left)
        let y: f32 = -((iy as f32 + ty) / height as f32 * 2.0 - 1.0);

        let ray = camera.shoot_at(x, y);

        if debug {
            color += &scene.color_debug(&ray);
        } else {
            // Shade the pixel with RGB color; 6 traces/reflections are made for
            // each intersection
            color += &scene.trace(&ray, 6);
        }
    }

    color *= 1.0 / AA_ITERATION_COUNT as f32;

    color.into()
}
