use color::{write_color, Color};
use std::io::{self, Write};

mod vec3;
mod color;

use vec3::Vec3;

fn main() -> io::Result<()> {

    const IMAGE_WIDTH: i32 = 256;
    const IMAGE_HEIGHT: i32 = 256;

    print!("P3\n{} {}\n255\n", IMAGE_WIDTH, IMAGE_HEIGHT);

    for j in (0..IMAGE_HEIGHT).rev() {
        write!(io::stderr(), "\rScanlines remaining: {} ", j)?;
        io::stderr().flush()?;

        for i in 0..IMAGE_WIDTH {
            let pixel_color = Color::new(
                (i as f64) / (IMAGE_WIDTH-1) as f64,
                (j as f64)/(IMAGE_HEIGHT-1) as f64,
                0.25,
            );
            write_color(io::stdout(), pixel_color)?;
        }
    }

    Ok(())
}
