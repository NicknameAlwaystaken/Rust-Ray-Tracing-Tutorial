use crate::vec3::Vec3;
use std::io::{self, Write};

pub type Color = Vec3;

pub fn write_color(mut out: impl Write, pixel_color: Color) -> io::Result<()> {
    writeln!(
        out,
        "{} {} {}",
        (255.999 * pixel_color.x) as i32,
        (255.999 * pixel_color.y) as i32,
        (255.999 * pixel_color.z) as i32,
    )
}
