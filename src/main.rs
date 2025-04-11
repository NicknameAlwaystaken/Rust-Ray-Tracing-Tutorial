use aarect::{XYRect, XZRect, YZRect};
use bvh::BvhNode;
use camera::Camera;
use color::write_color;
use constant_medium::ConstantMedium;
use cuboid::Cuboid;
use hittable::{FlipFace, Hittable, RotateY, Translate};
use material::{Dielectric, DiffuseLight, Lambertian, Material, Metal};
use moving_sphere::MovingSphere;
use rtweekend::{random_double, random_double_range, INFINITY};
use sphere::Sphere;
use texture::{CheckerTexture, ImageTexture, NoiseTexture, SolidColor, Texture};
use std::{io::{self, Write}, sync::{atomic::{AtomicI32, Ordering}, Arc}};
use rayon::prelude::*;

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
mod constant_medium;
mod onb;

use ray::Ray;
use vec3::{dot, Color, Point3, Vec3};

fn ray_color(r: &Ray, background: &Color, world: &Arc<dyn Hittable>, depth: u32) -> Color {

    // If we have exceeded the ray bounce limit, no more light is gathered.
    if depth == 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    if let Some(rec) = world.hit(&r, 0.001, INFINITY) {
        let emitted = rec.material.emitted(rec.u, rec.v, &rec.p, &rec);

        if let Some((albedo, _scattered, _pdf)) = rec.material.scatter(r, &rec) {
            let on_light = Point3::new(
                random_double_range(213.0, 343.0),
                554.0,
                random_double_range(227.0, 332.0),
            );

            let mut to_light = on_light - rec.p;
            let distance_squared = to_light.length_squared();
            to_light = to_light.unit_vector();

            if dot(&to_light, &rec.normal) < 0.0 {
                return emitted;
            }

            let light_area = (343.0 - 213.0) * (332.0 - 227.0);
            let light_cosine = to_light.y.abs();

            if light_cosine < 0.000001 {
                return emitted;
            }

            let pdf = distance_squared / (light_cosine * light_area);
            let scattered = Ray::with_time(rec.p, to_light, r.time);
            let scattering_pdf = rec.material.scattering_pdf(r, &rec, &scattered);

            return emitted
                + albedo
                * scattering_pdf
                * ray_color(&scattered, background, world, depth - 1)
                / pdf;
        }

        return emitted;
    }

    *background
}

pub fn final_scene() -> Arc<dyn Hittable> {
    let mut objects = vec![];

    // Ground: grid of boxes
    let mut boxes1: Vec<Arc<dyn Hittable>> = vec![];
    let ground: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Arc::new(SolidColor::new(Color::new(0.48, 0.83, 0.53))),
    });
    let boxes_per_side = 20;

    for i in 0..boxes_per_side {
        for j in 0..boxes_per_side {
            let w = 100.0;
            let x0 = -1000.0 + i as f64 * w;
            let z0 = -1000.0 + j as f64 * w;
            let y0 = 0.0;
            let x1 = x0 + w;
            let y1 = random_double_range(1.0, 101.0);
            let z1 = z0 + w;

            let cube = Arc::new(Cuboid::new(
                Point3::new(x0, y0, z0),
                Point3::new(x1, y1, z1),
                Arc::clone(&ground),
            ));
            boxes1.push(cube);
        }
    }

    objects.push(Arc::new(BvhNode::new(&mut boxes1, 0.0, 1.0)) as Arc<dyn Hittable>);

    // Light
    let light: Arc<dyn Material> = Arc::new(DiffuseLight {
        emit: Arc::new(SolidColor::new(Color::new(7.0, 7.0, 7.0))),
    });
    objects.push(Arc::new(XZRect::new(123.0, 423.0, 147.0, 412.0, 554.0, Arc::clone(&light))));

    // Moving sphere
    let center1 = Point3::new(400.0, 400.0, 200.0);
    let center2 = center1 + Vec3::new(30.0, 0.0, 0.0);
    let moving_mat: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Arc::new(SolidColor::new(Color::new(0.7, 0.3, 0.1))),
    });
    objects.push(Arc::new(MovingSphere::new(center1, center2, 0.0, 1.0, 50.0, moving_mat)));

    // Glass ball
    objects.push(Arc::new(Sphere::new(
        Point3::new(260.0, 150.0, 45.0),
        50.0,
        Arc::new(Dielectric::new(1.5)),
    )));

    // Metal ball
    objects.push(Arc::new(Sphere::new(
        Point3::new(0.0, 150.0, 145.0),
        50.0,
        Arc::new(Metal::new(Color::new(0.8, 0.8, 0.9), 1.0)),
    )));

    // Subsurface blue fog inside glass
    let boundary1: Arc<dyn Hittable> = Arc::new(Sphere::new(
        Point3::new(360.0, 150.0, 145.0),
        70.0,
        Arc::new(Dielectric::new(1.5)),
    ));
    objects.push(Arc::clone(&boundary1));
    objects.push(Arc::new(ConstantMedium::from_color(
        boundary1,
        0.2,
        Color::new(0.2, 0.4, 0.9),
    )));

    // Global white fog
    let boundary2 = Arc::new(Sphere::new(
        Point3::new(0.0, 0.0, 0.0),
        5000.0,
        Arc::new(Dielectric::new(1.5)),
    ));
    objects.push(Arc::new(ConstantMedium::from_color(
        boundary2,
        0.0001,
        Color::new(1.0, 1.0, 1.0),
    )));

    // Textured Earth sphere
    let earth_texture = Arc::new(ImageTexture::new("earthmap.jpg"));
    let earth_surface: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: earth_texture,
    });
    objects.push(Arc::new(Sphere::new(
        Point3::new(400.0, 200.0, 400.0),
        100.0,
        Arc::clone(&earth_surface),
    )));

    // Perlin noise sphere
    let noise_texture = Arc::new(NoiseTexture::new(0.1));
    let noise_material: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: noise_texture,
    });
    objects.push(Arc::new(Sphere::new(
        Point3::new(220.0, 280.0, 300.0),
        80.0,
        noise_material,
    )));

    // Small spheres cluster
    let mut boxes2: Vec<Arc<dyn Hittable>> = vec![];
    let white: Arc<dyn Material> = Arc::new(Lambertian {
        albedo: Arc::new(SolidColor::new(Color::new(0.73, 0.73, 0.73))),
    });

    for _ in 0..1000 {
        boxes2.push(Arc::new(Sphere::new(
            Point3::random_range(0.0, 165.0),
            10.0,
            Arc::clone(&white),
        )));
    }

    let cluster = Arc::new(Translate::new(
        Arc::new(RotateY::new(
            Arc::new(BvhNode::new(&mut boxes2, 0.0, 1.0)),
            15.0,
        )),
        Vec3::new(-100.0, 270.0, 395.0),
    ));
    objects.push(cluster);

    Arc::new(BvhNode::new(&mut objects, 0.0, 1.0))
}

pub fn cornell_smoke() -> Arc<dyn Hittable> {
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
    let smoke1 = Arc::new(ConstantMedium::from_color(box1, 0.01, Color::new(0.0, 0.0, 0.0)));
    objects.push(smoke1);


    let box2: Arc<dyn Hittable> = Arc::new(Cuboid::new(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 165.0, 165.0),
        Arc::clone(&white),
    ));
    let box2 = Arc::new(RotateY::new(box2, -18.0));
    let box2 = Arc::new(Translate::new(box2, Vec3::new(130.0, 0.0, 65.0)));
    let smoke2 = Arc::new(ConstantMedium::from_color(box2, 0.01, Color::new(1.0, 1.0, 1.0)));
    objects.push(smoke2);

    Arc::new(BvhNode::new(&mut objects, 0.0, 1.0))
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

    objects.push(Arc::new(
        FlipFace::new(Arc::new(
            XZRect::new(213.0, 343.0, 227.0, 332.0, 554.0, Arc::clone(&light))
        ))
    ));

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
            samples_per_pixel = 10;

            background = Color::new(0.0, 0.0, 0.0);
            lookfrom = Point3::new(278.0, 278.0, -800.0);
            lookat = Point3::new(278.0, 278.0, 0.0);
            vfov = 40.0;
        },
        7 => {
            world = cornell_smoke();

            aspect_ratio = 1.0;
            image_width = 600;
            samples_per_pixel = 200;

            background = Color::new(0.0, 0.0, 0.0);
            lookfrom = Point3::new(278.0, 278.0, -800.0);
            lookat = Point3::new(278.0, 278.0, 0.0);
            vfov = 40.0;
        },
        8 => {
            world = final_scene();

            aspect_ratio = 1.0;
            image_width = 800;
            samples_per_pixel = 10_000;

            background = Color::new(0.0, 0.0, 0.0);
            lookfrom = Point3::new(478.0, 278.0, -600.0);
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
    //
    let mut pixels: Vec<Color> = vec![Color::ZERO; (image_width * image_height) as usize];
    let remaining = Arc::new(AtomicI32::new(image_height));

    pixels
        .par_chunks_mut(image_width as usize)
        .enumerate()
        .for_each(|(j_rev, row)| {
            let j = image_height - 1 - j_rev as i32;
            for i in 0..image_width {
                let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                for _ in 0..samples_per_pixel {
                    let u = (i as f64 + random_double()) / (image_width - 1) as f64;
                    let v = (j as f64 + random_double()) / (image_height - 1) as f64;
                    let r = cam.get_ray(u, v);
                    pixel_color += ray_color(&r, &background, &world, MAX_DEPTH);
                }
                row[i as usize] = pixel_color;
            }
            let left = remaining.fetch_sub(1, Ordering::SeqCst);
            write!(io::stderr(), "\rScanlines remaining: {} ", left - 1).unwrap();
            io::stderr().flush().unwrap();
        });

    print!("P3\n{} {}\n255\n", image_width, image_height);

    for color in pixels {
        write_color(io::stdout(), color, samples_per_pixel)?;
    }

    Ok(())
}
