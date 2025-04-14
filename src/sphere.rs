use std::sync::Arc;

use crate::aabb::Aabb;
use crate::hittable::{HitRecord, Hittable};
use crate::material::Material;
use crate::onb::Onb;
use crate::ray::Ray;
use crate::rtweekend::{get_sphere_uv, random_double, random_double_range, INFINITY, PI};
use crate::vec3::{dot, Point3, Vec3};

pub struct Sphere {
    pub center: Point3,
    pub radius: f64,
    pub material: Arc<dyn Material>,
}

impl Sphere {
    pub fn new(center: Point3, radius: f64, material: Arc<dyn Material>) -> Self {
        Self {
            center,
            radius,
            material
        }
    }

    fn random_to_sphere(radius: f64, distance_squared: f64) -> Vec3 {
        let r1 = random_double();
        let r2 = random_double();
        let z = 1.0 + r2 * ((1.0 - radius * radius / distance_squared).sqrt() - 1.0);
        let phi = 2.0 * PI * r1;
        let x = phi.cos() * (1.0 - z*z).sqrt();
        let y = phi.sin() * (1.0 - z*z).sqrt();
        Vec3::new(x, y, z)
    }
}


impl Hittable for Sphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc: Vec3 = r.origin - self.center;
        let a = r.direction().length_squared();
        let half_b = dot(&oc, &r.direction());
        let c = oc.length_squared() - self.radius*self.radius;

        let discriminant = half_b*half_b - a*c;
        if discriminant < 0.0 {
            return None
        }

        let sqrtd = discriminant.sqrt();

        // Find the nearest root that lies in the acceptable range.
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || root > t_max {
            root = (-half_b + sqrtd) / a;
            if root < t_min || root > t_max {
                return None;
            }
        }

        let t = root;
        let p = r.at(root);
        let outward_normal = (p - self.center) / self.radius;
        let (u, v) = get_sphere_uv(&((p - self.center) / self.radius));

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

        Some(rec)
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
        Some(Aabb::new(
            self.center - Vec3::new(self.radius, self.radius, self.radius),
            self.center + Vec3::new(self.radius, self.radius, self.radius),
        ))
    }

    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f64 {
        let ray = Ray::with_time(*origin, *direction, 0.0);
        let rec = match self.hit(&ray, 0.001, INFINITY) {
            Some(rec) => rec,
            None => return 0.0,
        };

        let dist_sq = (self.center - origin).length_squared();
        let cos_theta_max = (1.0 - self.radius * self.radius / dist_sq).sqrt();
        let solid_angle = 2.0 * PI * (1.0 - cos_theta_max);

        1.0 / solid_angle
    }

    fn random(&self, origin: &Point3) -> Vec3 {
        let direction = self.center - origin;
        let distance_squared = direction.length_squared();
        let uvw = Onb::build_from_w(direction);
        uvw.local_vec(Self::random_to_sphere(self.radius, distance_squared))
    }

}
