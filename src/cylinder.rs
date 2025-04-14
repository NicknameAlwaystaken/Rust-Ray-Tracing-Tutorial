use std::sync::Arc;

use crate::{aabb::Aabb, hittable::{HitRecord, Hittable}, material::Material, ray::Ray, vec3::{Point3, Vec3}};



pub struct Cylinder {
    pub y0: f64,
    pub y1: f64,
    pub radius: f64,
    pub material: Arc<dyn Material>,
}

impl Cylinder {
    pub fn new(y0: f64, y1: f64, radius: f64, material: Arc<dyn Material>) -> Self {
        Self {
            y0,
            y1,
            radius,
            material,
        }
    }
}

enum HitSurface {
    Side,
    TopCap,
    BottomCap,
}


impl Hittable for Cylinder {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut hits: Vec<(f64, HitSurface)> = Vec::new();

        // check hit for top cap
        if let Some(t) = try_cap_hit(r, self.y1, self.radius, t_min, t_max) {
            hits.push((t, HitSurface::TopCap));
        }

        // check hit for bottom cap
        if let Some(t) = try_cap_hit(r, self.y0, self.radius, t_min, t_max) {
            hits.push((t, HitSurface::BottomCap));
        }

        if let Some(t) = try_side_hit(r, self.y0, self.y1, self.radius, t_min, t_max) {
            hits.push((t, HitSurface::Side));
        }

        let best_hit = hits
            .iter()
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        if let Some(&(t, ref surface)) = best_hit {
            let outward_normal;
            let p = r.at(t);

            match surface {
                HitSurface::TopCap => {
                    outward_normal = Vec3::new(0.0, 1.0, 0.0);
                }
                HitSurface::BottomCap => {
                    outward_normal = Vec3::new(0.0, -1.0, 0.0);
                }
                HitSurface::Side => {
                    outward_normal = Vec3::new(p.x, 0.0, p.z).unit_vector();
                }
            }

            let (u, v) = (0.0, 0.0);

            let mut rec = HitRecord {
                t,
                p,
                u,
                v,
                normal: Vec3::new(0.0, 0.0, 0.0),
                front_face: false,
                material: Arc::clone(&self.material),
            };
            rec.set_face_normal(r, outward_normal);

            return Some(rec)
        }
        None
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
        Some(Aabb::new(
            Point3::new(-self.radius, self.y0, -self.radius),
            Point3::new(self.radius, self.y1, self.radius),
        ))
    }
}

fn try_side_hit(r: &Ray, y0: f64, y1: f64, radius: f64, t_min: f64, t_max: f64) -> Option<f64>{
    let ox = r.origin.x;
    let oz = r.origin.z;

    let dx = r.direction.x;
    let dz = r.direction.z;

    let a = dx*dx +  dz*dz;
    let b = 2.0 * (ox * dx + oz * dz);
    let c = ox*ox +  oz*oz - radius * radius;

    let discriminant = b*b - 4.0*a*c;

    if discriminant < 0.0 {
        return None;
    }

    let sqrt_d = discriminant.sqrt();

    let mut t0 = (-b - sqrt_d) / (2.0 * a);
    let mut t1 = (-b + sqrt_d) / (2.0 * a);

    // check t values, starting from smaller, swap t values if t0 is not smaller

    if t0 > t1 {
        std::mem::swap(&mut t0, &mut t1);
    }

    let t = {
        let t_temp = t0;
        if t_temp >= t_min && t_temp <= t_max {

            let hit_y = r.origin.y + t_temp * r.direction.y;

            if hit_y >= y0 && hit_y <= y1 {
                return Some(t_temp);
            }
        }
        let t_temp = t1;

        if t_temp >= t_min && t_temp <= t_max {

            let hit_y = r.origin.y + t_temp * r.direction.y;

            if hit_y >= y0 && hit_y <= y1 {
                return Some(t_temp);
            }
        }

        None
    };

    t
}

fn try_cap_hit(r: &Ray, y: f64, radius: f64, t_min: f64, t_max: f64) -> Option<f64>{
    let t = (y - r.origin.y) / r.direction.y;
    if t < t_min || t > t_max {
        return None;
    }

    let x = r.origin.x + t * r.direction.x;
    let z = r.origin.z + t * r.direction.z;

    // epsilon accounting for floating point precision errors
    if x*x + z*z > radius*radius + 1e-8 {
        return None;
    }

    Some(t)
}
