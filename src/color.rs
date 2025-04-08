use crate::{rtweekend::clamp, vec3::Color};
use std::io::{self, Write};

pub fn write_color(mut out: impl Write, pixel_color: Color, samples_per_pixel: u32) -> io::Result<()> {
    let mut r = pixel_color.x;
    let mut g = pixel_color.y;
    let mut b = pixel_color.z;

    // Divide the color by number of samples and gamma-correct for gamma=2.0.
    let scale = 1.0 / samples_per_pixel as f64;

    r = (scale * r).sqrt();
    g = (scale * g).sqrt();
    b = (scale * b).sqrt();

    writeln!(
        out,
        "{} {} {}",
        (256.0 * clamp(r, 0.0, 0.999)) as i32,
        (256.0 * clamp(g, 0.0, 0.999)) as i32,
        (256.0 * clamp(b, 0.0, 0.999)) as i32,
    )
}
