use color::write_color;
use std::io::{self, Write};

mod vec3;
mod color;
mod ray;

use ray::Ray;
use vec3::{Vec3, dot, unit_vector, Color, Point3};

fn ray_color(r: &Ray) -> Color {
    if hit_sphere(&Point3::new(0.0, 0.0, -1.0), 0.5, r) {
        return Color::new(1.0, 0.0, 0.0);
    }
    let unit_direction: Vec3 = unit_vector(&r.direction);
    let t: f64 = 0.5*(unit_direction.y + 1.0);
    Color::new(1.0, 1.0, 1.0)*(1.0-t) + Color::new(0.5, 0.7, 1.0)*t
}

fn hit_sphere(center: &Point3, radius: f64, r: &Ray) -> bool {
    let oc: Vec3 = r.origin - *center;
    let a = dot(&r.direction(), &r.direction());
    let b = 2.0 * dot(&oc, &r.direction());
    let c = dot(&oc, &oc) - radius*radius;
    let discriminant = b*b - 4.0*a*c;
    discriminant > 0.0
}

fn main() -> io::Result<()> {

    const ASPECT_RATIO: f32 = 16.0/9.0;

    const IMAGE_WIDTH: i32 = 400;
    const IMAGE_HEIGHT: i32 = (IMAGE_WIDTH as f32 / ASPECT_RATIO) as i32;

    let viewport_height: f64 = 2.0;
    let viewport_width: f64 = (ASPECT_RATIO as f64) * viewport_height;
    let focal_length: f64 = 1.0;

    let origin = Point3::new(0.0, 0.0, 0.0);
    let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
    let vertical = Vec3::new(0.0, viewport_height, 0.0);
    let lower_left_corner = origin - horizontal/2.0 - vertical/2.0 - Vec3::new(0.0, 0.0, focal_length);

    print!("P3\n{} {}\n255\n", IMAGE_WIDTH, IMAGE_HEIGHT);

    for j in (0..IMAGE_HEIGHT).rev() {
        write!(io::stderr(), "\rScanlines remaining: {} ", j)?;
        io::stderr().flush()?;
        for i in 0..IMAGE_WIDTH {
            let u = (i as f64) / (IMAGE_WIDTH-1) as f64;
            let v = (j as f64) / (IMAGE_HEIGHT-1) as f64;
            let r: Ray = Ray::new(origin, lower_left_corner + u*horizontal + v*vertical - origin);
            let pixel_color: Color = ray_color(&r);
            write_color(io::stdout(), pixel_color)?;
        }
    }

    Ok(())
}
