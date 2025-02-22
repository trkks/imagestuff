pub fn main() {
    let width = 256;
    let height = 256;
    println!("P3");
    println!("{} {}", width, height);
    println!("255");
    for j in 0..height {
        for i in 0..width {
            let color = raycast::shade(width, height, i, j);
            let [r, g, b] = color.into();
            println!("{} {} {}", r, g, b);
        }
    }
}
