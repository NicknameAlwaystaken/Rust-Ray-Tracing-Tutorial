use std::sync::Arc;

use crate::material::Material;
use crate::ray::Ray;
use crate::rtweekend::{degrees_to_radians, INFINITY};
use crate::vec3::{dot, Point3, Vec3};
use crate::aabb::Aabb;

pub trait Hittable {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb>;
}

pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub material: Arc<dyn Material>,
    pub t: f64,
    pub u: f64,
    pub v: f64,
    pub front_face: bool,
}

pub struct Translate {
    pub ptr: Arc<dyn Hittable>,
    pub offset: Vec3,
}

pub struct RotateY {
    pub ptr: Arc<dyn Hittable>,
    pub sin_theta: f64,
    pub cos_theta: f64,
    pub hasbox: bool,
    pub bbox: Aabb,
}

impl RotateY {
    pub fn new(ptr: Arc<dyn Hittable>, angle: f64) -> Self {
        let radians = degrees_to_radians(angle);
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();

        let mut bbox = Aabb::default();
        let hasbox = ptr.bounding_box(0.0, 1.0).map(|b| {
            bbox = b;

            let mut min = Point3::new(INFINITY, INFINITY, INFINITY);
            let mut max = Point3::new(-INFINITY, -INFINITY, -INFINITY);

            for i in 0..2 {
                let x = if i == 0 { bbox.minimum.x } else { bbox.maximum.x };
                for j in 0..2 {
                    let y = if j == 0 { bbox.minimum.y } else { bbox.maximum.y };
                    for k in 0..2 {
                        let z = if k == 0 { bbox.minimum.z } else { bbox.maximum.z };

                        let newx = cos_theta * x + sin_theta * z;
                        let newz = -sin_theta * x + cos_theta * z;

                        let tester = Vec3::new(newx, y, newz);

                        min.x = min.x.min(tester.x);
                        min.y = min.y.min(tester.y);
                        min.z = min.z.min(tester.z);

                        max.x = max.x.max(tester.x);
                        max.y = max.y.max(tester.y);
                        max.z = max.z.max(tester.z);
                    }
                }
            }
            bbox = Aabb::new(min, max);
            true
        }).unwrap_or(false);

        Self {
            ptr,
            sin_theta,
            cos_theta,
            hasbox,
            bbox,
        }
    }
}

impl HitRecord {
    pub fn set_face_normal(&mut self, ray: &Ray, outward_normal: Vec3) {
        self.front_face = dot(&ray.direction, &outward_normal) < 0.0;
        self.normal = if self.front_face { outward_normal } else { -outward_normal };
    }
}

impl Translate {
    pub fn new(ptr: Arc<dyn Hittable>, offset: Vec3) -> Self {
        Self { ptr, offset }
    }
}

impl Hittable for Translate {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let moved_r = Ray::with_time(r.origin - self.offset, r.direction, r.time);

        if let Some(mut rec) = self.ptr.hit(&moved_r, t_min, t_max) {
            rec.p += self.offset;
            rec.set_face_normal(&moved_r, rec.normal);
            Some(rec)
        } else {
            None
        }
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
        self.ptr.bounding_box(time0, time1).map(|bbox| {
            Aabb::new(bbox.minimum + self.offset, bbox.maximum + self.offset)
        })
    }
}

impl Hittable for RotateY {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let origin = {
            let x = self.cos_theta * r.origin.x - self.sin_theta * r.origin.z;
            let z = self.sin_theta * r.origin.x + self.cos_theta * r.origin.z;
            Point3::new(x, r.origin.y, z)
        };

        let direction = {
            let x = self.cos_theta * r.direction.x - self.sin_theta * r.direction.z;
            let z = self.sin_theta * r.direction.x + self.cos_theta * r.direction.z;
            Vec3::new(x, r.direction.y, z)
        };

        let rotated_r = Ray::with_time(origin, direction, r.time);

        // Hit test in rotated space
        if let Some(mut rec) = self.ptr.hit(&rotated_r, t_min, t_max) {
            // Rotate hit point and normal back to world space
            let p = {
                let x = self.cos_theta * rec.p.x + self.sin_theta * rec.p.z;
                let z = -self.sin_theta * rec.p.x + self.cos_theta * rec.p.z;
                Point3::new(x, rec.p.y, z)
            };

            let normal = {
                let x = self.cos_theta * rec.normal.x + self.sin_theta * rec.normal.z;
                let z = -self.sin_theta * rec.normal.x + self.cos_theta * rec.normal.z;
                Vec3::new(x, rec.normal.y, z)
            };

            rec.p = p;

            rec.set_face_normal(&rotated_r, normal);

            Some(rec)

        } else {
            None
        }
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
        if self.hasbox {
            Some(self.bbox)
        } else {
            None
        }
    }
}
