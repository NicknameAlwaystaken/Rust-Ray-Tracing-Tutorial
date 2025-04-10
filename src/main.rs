use bvh::BvhNode;
use camera::Camera;
use color::write_color;
use hittable::Hittable;
use material::{Dielectric, Lambertian, Material, Metal};
use moving_sphere::MovingSphere;
use rtweekend::{random_double, random_double_range, INFINITY};
use sphere::Sphere;
use texture::{CheckerTexture, NoiseTexture, SolidColor};
use std::{io::{self, Write}, sync::Arc};

mod vec3;
mod color;
mod ray;
mod hittable;
mod sphere;
mod hittable_list;
mod rtweekend;
mod camera;
mod material;
mod moving_sphere;
mod aabb;
mod bvh;
mod texture;
mod perlin;

use ray::Ray;
use vec3::{dot, Color, Point3, Vec3};

fn ray_color(r: &Ray, world: &Arc<dyn Hittable>, depth: u32) -> Color {

    // If we have exceeded the ray bounce limit, no more light is gathered.
    if depth == 0 {
        return Color::new(0.0, 0.0, 0.0);
    }
    if let Some(rec) = world.hit(&r, 0.001, INFINITY) {
        if let Some((attentuation, scattered)) = rec.material.scatter(r, &rec) {
            return attentuation * ray_color(&scattered, world, depth - 1);
        } else {
            return Color::new(0.0, 0.0, 0.0);
        }
    }

    let unit_direction = r.direction.unit_vector();
    let t = 0.5 * (unit_direction.y + 1.0);
    (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
}

pub fn two_perlin_spheres() -> Arc<dyn Hittable> {
    let mut objects: Vec<Arc<dyn Hittable>> = vec![];

    let pertext = Arc::new(NoiseTexture::new(4.0));

    let pertext_material: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: pertext,
    });

    objects.push(Arc::new(Sphere {
        center: Point3::new(0.0, -1000.0, 0.0),
        radius: 1000.0,
        material: Arc::clone(&pertext_material),
    }));

    objects.push(Arc::new(Sphere {
        center: Point3::new(0.0, 2.0, 0.0),
        radius: 2.0,
        material: Arc::clone(&pertext_material),
    }));

    Arc::new(BvhNode::new(&mut objects, 0.0, 1.0))
}

pub fn two_spheres() -> Arc<dyn Hittable> {
    let mut objects: Vec<Arc<dyn Hittable>> = vec![];

    let checker = Arc::new(CheckerTexture::from_colors(
        Color::new(0.2, 0.3, 0.1),
        Color::new(0.9, 0.9, 0.9),
    ));

    let checker_material: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: checker,
    });

    objects.push(Arc::new(Sphere {
        center: Point3::new(0.0, -10.0, 0.0),
        radius: 10.0,
        material: Arc::clone(&checker_material),
    }));

    objects.push(Arc::new(Sphere {
        center: Point3::new(0.0, 10.0, 0.0),
        radius: 10.0,
        material: Arc::clone(&checker_material),
    }));

    Arc::new(BvhNode::new(&mut objects, 0.0, 1.0))
}

pub fn random_scene() -> Arc<dyn Hittable> {
    let mut objects: Vec<Arc<dyn Hittable>> = vec![];

    let checker = Arc::new(CheckerTexture::from_colors(
        Color::new(0.2, 0.3, 0.1),
        Color::new(0.9, 0.9, 0.9),
    ));

    let checker_material = Arc::new(Lambertian {
        albedo: checker,
    });

    objects.push(Arc::new(Sphere {
        center: Point3::new(0.0, -1000.0, 0.0),
        radius: 1000.0,
        material: checker_material,
    }));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = random_double();
            let center = Point3::new(
                a as f64 + 0.9 * random_double(),
                0.2,
                b as f64 + 0.9 * random_double(),
            );

            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                let sphere_material: Arc<dyn Material>;

                if choose_mat < 0.8 {
                    // Diffuse
                    let albedo = Color::random() * Color::random();
                    sphere_material = Arc::new(Lambertian {
                        albedo: Arc::new(SolidColor::new(albedo)),
                    });
                    let center2 = center + Vec3::new(0.0, random_double_range(0.0, 0.5), 0.0);
                    objects.push(Arc::new(MovingSphere {
                        center0: center,
                        center1: center2,
                        time0: 0.0,
                        time1: 1.0,
                        radius: 0.2,
                        material: Arc::clone(&sphere_material),
                    }));
                } else if choose_mat < 0.95 {
                    let albedo = Color::random_range(0.5, 1.0);
                    let fuzz = random_double_range(0.0, 0.5);
                    sphere_material = Arc::new(Metal { albedo, fuzz});

                    objects.push(Arc::new(Sphere {
                        center,
                        radius: 0.2,
                        material: sphere_material,
                    }));
                } else {
                    // Glass
                    sphere_material = Arc::new(Dielectric { ir: 1.5 });

                    objects.push(Arc::new(Sphere {
                        center,
                        radius: 0.2,
                        material: sphere_material,
                    }));
                }
            }
        }
    }

    // Three big spheres
    let material1: Arc<dyn Material> = Arc::new(Dielectric { ir: 1.5 });
    objects.push(Arc::new(Sphere {
        center: Point3::new(0.0, 1.0, 0.0),
        radius: 1.0,
        material: material1,
    }));

    let material2: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Arc::new(SolidColor::new(Color::new(0.4, 0.2, 0.1))),
    });
    objects.push(Arc::new(Sphere {
        center: Point3::new(-4.0, 1.0, 0.0),
        radius: 1.0,
        material: material2,
    }));

    let material3: Arc<dyn Material> = Arc::new(Metal {
        albedo: Color::new(0.7, 0.6, 0.5),
        fuzz: 0.0,
    });
    objects.push(Arc::new(Sphere {
        center: Point3::new(4.0, 1.0, 0.0),
        radius: 1.0,
        material: material3,
    }));

    Arc::new(BvhNode::new(&mut objects, 0.0, 1.0))
}

fn main() -> io::Result<()> {

    // Image
    const ASPECT_RATIO: f64 = 16.0/9.0;
    const IMAGE_WIDTH: i32 = 400;
    const IMAGE_HEIGHT: i32 = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as i32;
    const SAMPLES_PER_PIXEL: u32 = 100;
    const MAX_DEPTH: u32 = 50;

    let lookfrom: Point3;
    let lookat: Point3;
    let mut vfov = 40.0;
    let mut aperture = 0.0;

    let world: Arc<dyn Hittable>;

    let scene_id = 3;

    match scene_id {
        1 => {

            world = random_scene();

            lookfrom = Point3::new(13.0, 2.0, 3.0);
            lookat = Point3::new(0.0, 0.0, 0.0);
            vfov = 20.0;
            aperture = 0.1;
        },
        2 => {

            world = two_spheres();

            lookfrom = Point3::new(13.0, 2.0, 3.0);
            lookat = Point3::new(0.0, 0.0, 0.0);
            vfov = 20.0;
        },
        3 => {
            world = two_perlin_spheres();

            lookfrom = Point3::new(13.0, 2.0, 3.0);
            lookat = Point3::new(0.0, 0.0, 0.0);
            vfov = 20.0;
        }
        _ => {
            world = random_scene();

            lookfrom = Point3::new(13.0, 2.0, 3.0);
            lookat = Point3::new(0.0, 0.0, 0.0);
            vfov = 20.0;
            aperture = 0.1;
        }
    }

    let vup = Vec3::new(0.0, 1.0, 0.0);
    let dist_to_focus = 10.0;

    let cam: Camera = Camera::new(
        lookfrom,
        lookat,
        vup,
        vfov,
        ASPECT_RATIO,
        aperture,
        dist_to_focus,
        0.0,
        1.0
    );

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
                pixel_color += ray_color(&r, &world, MAX_DEPTH);
            }

            write_color(io::stdout(), pixel_color, SAMPLES_PER_PIXEL)?;
        }
    }

    Ok(())
}
