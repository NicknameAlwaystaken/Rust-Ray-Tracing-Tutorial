use aarect::{XYRect, XZRect, YZRect};
use bvh::BvhNode;
use camera::Camera;
use color::write_color;
use cuboid::Cuboid;
use hittable::{Hittable, RotateY, Translate};
use material::{Dielectric, DiffuseLight, Lambertian, Material, Metal};
use moving_sphere::MovingSphere;
use rtweekend::{random_double, random_double_range, INFINITY};
use sphere::Sphere;
use texture::{CheckerTexture, ImageTexture, NoiseTexture, SolidColor, Texture};
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
mod aarect;
mod cuboid;

use ray::Ray;
use vec3::{dot, Color, Point3, Vec3};

fn ray_color(r: &Ray, background: &Color, world: &Arc<dyn Hittable>, depth: u32) -> Color {

    // If we have exceeded the ray bounce limit, no more light is gathered.
    if depth == 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    if let Some(rec) = world.hit(&r, 0.001, INFINITY) {
        let emitted = rec.material.emitted(rec.u, rec.v, &rec.p);

        if let Some((attentuation, scattered)) = rec.material.scatter(r, &rec) {
            return emitted + attentuation * ray_color(&scattered, background, world, depth - 1);
        } else {
            return emitted;
        }
    }

    *background
}

pub fn cornell_box() -> Arc<dyn Hittable> {
    let mut objects: Vec<Arc<dyn Hittable>> = vec![];

    let red: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Arc::new(SolidColor::new(Color::new(0.65, 0.05, 0.05))),
    });

    let white: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Arc::new(SolidColor::new(Color::new(0.73, 0.73, 0.73))),
    });

    let green: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Arc::new(SolidColor::new(Color::new(0.12, 0.45, 0.15))),
    });

    let light: Arc<dyn Material> = Arc::new(DiffuseLight {
        emit: Arc::new(SolidColor::new(Color::new(15.0, 15.0, 15.0))),
    });

    objects.push(Arc::new(YZRect::new(0.0, 555.0, 0.0, 555.0, 555.0, Arc::clone(&green))));
    objects.push(Arc::new(YZRect::new(0.0, 555.0, 0.0, 555.0, 0.0, Arc::clone(&red))));

    objects.push(Arc::new(XZRect::new(213.0, 343.0, 227.0, 332.0, 554.0, Arc::clone(&light))));

    objects.push(Arc::new(XZRect::new(0.0, 555.0, 0.0, 555.0, 0.0, Arc::clone(&white))));
    objects.push(Arc::new(XZRect::new(0.0, 555.0, 0.0, 555.0, 555.0, Arc::clone(&white))));
    objects.push(Arc::new(XYRect::new(0.0, 555.0, 0.0, 555.0, 555.0, Arc::clone(&white))));

    // two boxes

    let box1: Arc<dyn Hittable> = Arc::new(Cuboid::new(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 330.0, 165.0),
        Arc::clone(&white),
    ));
    let box1 = Arc::new(RotateY::new(box1, 15.0));
    let box1 = Arc::new(Translate::new(box1, Vec3::new(265.0, 0.0, 295.0)));
    objects.push(box1);


    let box2: Arc<dyn Hittable> = Arc::new(Cuboid::new(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 165.0, 165.0),
        Arc::clone(&white),
    ));
    let box2 = Arc::new(RotateY::new(box2, -18.0));
    let box2 = Arc::new(Translate::new(box2, Vec3::new(130.0, 0.0, 65.0)));
    objects.push(box2);

    Arc::new(BvhNode::new(&mut objects, 0.0, 1.0))
}

pub fn earth() -> Arc<dyn Hittable> {
    let earth_texture: Arc<dyn Texture> = Arc::new(ImageTexture::new("earthmap.jpg"));
    let earth_surface: Arc<dyn Material> = Arc::new(Lambertian { albedo: earth_texture });
    let globe: Arc<dyn Hittable> = Arc::new(Sphere {
        center: Point3::new(0.0, 0.0, 0.0),
        radius: 2.0,
        material: earth_surface,
    });

    Arc::new(BvhNode::new(&mut vec![globe], 0.0, 1.0))
}

/*
pub fn simple_light() -> Arc<dyn Hittable> {
    let mut objects: Vec<Arc<dyn Hittable>> = vec![];

    let pertext = Arc::new(NoiseTexture::new(4.0));
    let lambert: Arc<dyn Material> = Arc::new(Lambertian { albedo: pertext });

    objects.push(Arc::new(Sphere {
        center: Point3::new(0.0, -1000.0, 0.0),
        radius: 1000.0,
        material: Arc::clone(&lambert),
    }));

    objects.push(Arc::new(Sphere {
        center: Point3::new(0.0, 2.0, 0.0),
        radius: 2.0,
        material: Arc::clone(&lambert),
    }));

    let light: Arc<dyn Material> = Arc::new(DiffuseLight {
        emit: Arc::new(SolidColor::new(Color::new(4.0, 4.0, 4.0))),
    });

    objects.push(Arc::new(XYRect::new(3.0, 5.0, 1.0, 3.0, -2.0, light)));

    Arc::new(BvhNode::new(&mut objects, 0.0, 1.0))
}
*/

pub fn simple_light() -> Arc<dyn Hittable> {
    let mut objects: Vec<Arc<dyn Hittable>> = vec![];

    let pertext = Arc::new(NoiseTexture::new(4.0));
    let lambert: Arc<dyn Material> = Arc::new(Lambertian { albedo: pertext });

    objects.push(Arc::new(Sphere {
        center: Point3::new(0.0, -1000.0, 0.0),
        radius: 1000.0,
        material: Arc::clone(&lambert),
    }));

    objects.push(Arc::new(Sphere {
        center: Point3::new(0.0, 2.0, 0.0),
        radius: 2.0,
        material: Arc::clone(&lambert),
    }));

    let light_color = Arc::new(SolidColor::new(Color::new(4.0, 4.0, 4.0)));
    let light: Arc<dyn Material> = Arc::new(DiffuseLight { emit: light_color });

    objects.push(Arc::new(XYRect::new(3.0, 5.0, 3.0, 5.0, -2.0, Arc::clone(&light))));

    // Glowing sphere nearly under main sphere
    objects.push(Arc::new(Sphere {
        center: Point3::new(2.0, 0.5, -0.2),
        radius: 0.5,
        material: Arc::clone(&light),
    }));

    // Glowing ball on the left
    objects.push(Arc::new(Sphere {
        center: Point3::new(-3.5, 0.5, 2.0),
        radius: 0.5,
        material: Arc::clone(&light),
    }));

    Arc::new(BvhNode::new(&mut objects, 0.0, 1.0))
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
    let mut aspect_ratio: f64 = 16.0/9.0;
    let mut image_width: i32 = 400;
    const MAX_DEPTH: u32 = 50;

    let mut samples_per_pixel: u32 = 100;

    let lookfrom: Point3;
    let lookat: Point3;
    let mut vfov = 40.0;
    let mut aperture = 0.0;
    let mut background = Color::new(0.0, 0.0, 0.0);

    let world: Arc<dyn Hittable>;

    let scene_id = 6;

    match scene_id {
        1 => {

            world = random_scene();

            background = Color::new(0.70, 0.80, 1.00);
            lookfrom = Point3::new(13.0, 2.0, 3.0);
            lookat = Point3::new(0.0, 0.0, 0.0);
            vfov = 20.0;
            aperture = 0.1;
        },
        2 => {

            world = two_spheres();

            background = Color::new(0.70, 0.80, 1.00);
            lookfrom = Point3::new(13.0, 2.0, 3.0);
            lookat = Point3::new(0.0, 0.0, 0.0);
            vfov = 20.0;
        },
        3 => {
            world = two_perlin_spheres();

            background = Color::new(0.70, 0.80, 1.00);
            lookfrom = Point3::new(13.0, 2.0, 3.0);
            lookat = Point3::new(0.0, 0.0, 0.0);
            vfov = 20.0;
        },
        4 => {
            world = earth();
            background = Color::new(0.70, 0.80, 1.00);
            lookfrom = Point3::new(13.0, 2.0, 3.0);
            lookat = Point3::new(0.0, 0.0, 0.0);
            vfov = 20.0;
        },
        5 => {
            world = simple_light();

            samples_per_pixel = 400;
            background = Color::new(0.0, 0.0, 0.0);
            lookfrom = Point3::new(26.0, 3.0, 6.0);
            lookat = Point3::new(0.0, 2.0, 0.0);
            vfov = 20.0;
            aperture = 0.1;
        },
        6 => {
            world = cornell_box();

            aspect_ratio = 1.0;
            image_width = 600;
            samples_per_pixel = 200;

            background = Color::new(0.0, 0.0, 0.0);
            lookfrom = Point3::new(278.0, 278.0, -800.0);
            lookat = Point3::new(278.0, 278.0, 0.0);
            vfov = 40.0;
        },
        _ => {
            world = random_scene();

            background = Color::new(0.70, 0.80, 1.00);
            lookfrom = Point3::new(13.0, 2.0, 3.0);
            lookat = Point3::new(0.0, 0.0, 0.0);
            vfov = 20.0;
            aperture = 0.1;
        }
    }

    let vup = Vec3::new(0.0, 1.0, 0.0);
    let dist_to_focus = 10.0;
    let image_height: i32 = (image_width as f64 / aspect_ratio) as i32;

    let cam: Camera = Camera::new(
        lookfrom,
        lookat,
        vup,
        vfov,
        aspect_ratio,
        aperture,
        dist_to_focus,
        0.0,
        1.0
    );

    // Render

    print!("P3\n{} {}\n255\n", image_width, image_height);

    for j in (0..image_height).rev() {
        write!(io::stderr(), "\rScanlines remaining: {} ", j)?;
        io::stderr().flush()?;
        for i in 0..image_width {
            let mut pixel_color: Color = Color::new(0.0, 0.0, 0.0);

            for _ in 0..samples_per_pixel {
                let u = (i as f64 + random_double()) / (image_width-1) as f64;
                let v = (j as f64 + random_double()) / (image_height-1) as f64;
                let r: Ray = cam.get_ray(u, v);
                pixel_color += ray_color(&r, &background, &world, MAX_DEPTH);
            }

            write_color(io::stdout(), pixel_color, samples_per_pixel)?;
        }
    }

    Ok(())
}
