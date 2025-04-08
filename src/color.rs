use crate::{rtweekend::clamp, vec3::Vec3};
use std::io::{self, Write};

pub type Color = Vec3;

pub fn write_color(mut out: impl Write, pixel_color: Color, samples_per_pixel: i32) -> io::Result<()> {
    let mut r = pixel_color.x;
    let mut g = pixel_color.y;
    let mut b = pixel_color.z;

    // Divide the color by number of samples.
    let scale = 1.0 / samples_per_pixel as f64;

    r *= scale;
    g *= scale;
    b *= scale;

    writeln!(
        out,
        "{} {} {}",
        (256.0 * clamp(r, 0.0, 0.999)) as i32,
        (256.0 * clamp(g, 0.0, 0.999)) as i32,
        (256.0 * clamp(b, 0.0, 0.999)) as i32,
    )
}
