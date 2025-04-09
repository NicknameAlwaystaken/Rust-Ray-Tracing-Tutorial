use std::sync::Arc;

use crate::{aabb::Aabb, hittable::{HitRecord, Hittable}, material::Material, ray::Ray, rtweekend::get_sphere_uv, vec3::{dot, Point3, Vec3}};

pub struct MovingSphere {
    pub center0: Point3,
    pub center1: Point3,
    pub time0: f64,
    pub time1: f64,
    pub radius: f64,
    pub material: Arc<dyn Material>,
}

impl MovingSphere {
    pub fn new(
        center0: Point3,
        center1: Point3,
        time0: f64,
        time1: f64,
        radius: f64,
        material: Arc<dyn Material>,
    ) -> Self {

        Self { center0, center1, time0, time1, radius, material }
    }

    pub fn center(&self, time: f64) -> Point3 {
        self.center0 + ((time - self.time0) / (self.time1 - self.time0)) * (self.center1 - self.center0)
    }
}

impl Hittable for MovingSphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc: Vec3 = r.origin - self.center(r.time());
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
        let outward_normal = (p - self.center(r.time())) / self.radius;
        let (u, v) = get_sphere_uv(&((p - self.center(r.time())) / self.radius));

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

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<crate::aabb::Aabb> {
        let radius_vec = Vec3::new(self.radius, self.radius, self.radius);

        let box0 = Aabb::new(
            self.center(time0) - radius_vec,
            self.center(time0) + radius_vec,
        );

        let box1 = Aabb::new(
            self.center(time1) - radius_vec,
            self.center(time1) + radius_vec,
        );
        Some(Aabb::surrounding_box(&box0, &box1))
    }
}
