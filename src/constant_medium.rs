use std::sync::Arc;

use crate::{hittable::{HitRecord, Hittable}, material::{Isotropic, Material}, ray::Ray, rtweekend::{random_double, INFINITY}, texture::{SolidColor, Texture}, vec3::{Color, Vec3}};




pub struct ConstantMedium {
    pub boundary: Arc<dyn Hittable>,
    pub phase_function: Arc<dyn Material>,
    pub neg_inv_density: f64,
}

impl ConstantMedium {
    pub fn from_texture(boundary: Arc<dyn Hittable>, density: f64, texture: Arc<dyn Texture>) -> Self {
        Self {
            boundary,
            phase_function: Arc::new(Isotropic { albedo: texture }),
            neg_inv_density: -1.0 / density,
        }
    }

    pub fn from_color(boundary: Arc<dyn Hittable>, density: f64, color: Color) -> Self {
        let texture = Arc::new(SolidColor::new(color));
        Self::from_texture(boundary, density, texture)
    }
}

impl Hittable for ConstantMedium {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut rec1 = self.boundary.hit(r, -INFINITY, INFINITY)?;
        let mut rec2 = self.boundary.hit(r, rec1.t + 0.0001, INFINITY)?;

        if rec1.t < t_min {
            rec1.t = t_min;
        }
        if rec2.t > t_max {
            rec2.t = t_max;
        }

        if rec1.t >= rec2.t {
            return None;
        }

        if rec1.t < 0.0 {
            rec1.t = 0.0;
        }

        let ray_length = r.direction.length();
        let distance_inside_boundary = (rec2.t - rec1.t) * ray_length;
        let hit_distance = self.neg_inv_density * random_double().ln();

        if hit_distance > distance_inside_boundary {
            return None;
        }

        let t = rec1.t + hit_distance / ray_length;
        let p = r.at(t);

        Some(HitRecord {
            t,
            p,
            normal: Vec3::new(1.0, 0.0, 0.0),
            front_face: true,
            material: Arc::clone(&self.phase_function),
            u: 0.0,
            v: 0.0,
        })
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<crate::aabb::Aabb> {
        self.boundary.bounding_box(time0, time1)
    }
}
