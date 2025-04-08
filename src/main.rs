use camera::Camera;
use color::write_color;
use hittable::{HitRecord, Hittable};
use hittable_list::HittableList;
use rtweekend::{random_double, INFINITY};
use sphere::Sphere;
use std::io::{self, Write};

mod vec3;
mod color;
mod ray;
mod hittable;
mod sphere;
mod hittable_list;
mod rtweekend;
mod camera;

use ray::Ray;
use vec3::{dot, unit_vector, Color, Point3, Vec3};

fn ray_color(r: &Ray, world: &dyn Hittable) -> Color {
    if let Some(rec) = world.hit(&r, 0.0, INFINITY) {
        return 0.5 * (rec.normal + Color::new(1.0, 1.0, 1.0));
    }

    let unit_direction = r.direction.unit_vector();
    let t = 0.5 * (unit_direction.y + 1.0);
    (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
}

fn hit_sphere(center: &Point3, radius: f64, r: &Ray) -> f64 {
    let oc: Vec3 = r.origin - *center;
    let a = r.direction().length_squared();
    let half_b = dot(&oc, &r.direction());
    let c = oc.length_squared() - radius*radius;
    let discriminant = half_b*half_b - a*c;

    if discriminant < 0.0 {
        -1.0
    } else {
        (-half_b - discriminant.sqrt() ) / a
    }
}

fn main() -> io::Result<()> {

    // Image
    const ASPECT_RATIO: f32 = 16.0/9.0;
    const IMAGE_WIDTH: i32 = 400;
    const IMAGE_HEIGHT: i32 = (IMAGE_WIDTH as f32 / ASPECT_RATIO) as i32;
    const SAMPLES_PER_PIXEL: i32 = 100;

    // World
    let mut world = HittableList::new();
    world.add(Box::new(
        Sphere {
            center: Point3::new(0.0, 0.0, -1.0),
            radius: 0.5
        }));
    world.add(Box::new(
        Sphere {
            center: Point3::new(0.0, -100.5, -1.0),
            radius: 100.0,
        }));

    // Camera
    let cam: Camera = Camera::new();

    // Render

    print!("P3\n{} {}\n255\n", IMAGE_WIDTH, IMAGE_HEIGHT);

    for j in (0..IMAGE_HEIGHT).rev() {
        write!(io::stderr(), "\rScanlines remaining: {} ", j)?;
        io::stderr().flush()?;
        for i in 0..IMAGE_WIDTH {
            let mut pixel_color: Color = Color::new(0.0, 0.0, 0.0);

            for _ in 0..SAMPLES_PER_PIXEL {
                let u = (i as f64 + random_double()) / (IMAGE_WIDTH-1) as f64;
                let v = (j as f64 + random_double()) / (IMAGE_HEIGHT-1) as f64;
                let r: Ray = cam.get_ray(u, v);
                pixel_color += ray_color(&r, &world);
            }

            write_color(io::stdout(), pixel_color, SAMPLES_PER_PIXEL)?;
        }
    }

    Ok(())
}
