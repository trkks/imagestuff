mod color;
mod math;

pub fn shade(w: u32, h: u32, x: u32, y: u32) -> color::Color {
    let r = (x as f32) / ((w - 1) as f32);
    let g = (y as f32) / ((h - 1) as f32);
    let b = 0;
    color::Color::new(r, g, b as f32)
}
