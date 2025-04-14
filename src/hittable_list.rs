use std::sync::Arc;

use rand::{distr::weighted::Weight, random, seq::IndexedRandom};

use crate::{aabb::Aabb, hittable::{HitRecord, Hittable}, ray::Ray, rtweekend::random_int, vec3::{Point3, Vec3}};

pub struct HittableList {
    pub objects: Vec<Arc<dyn Hittable>>,
}

impl HittableList {
    pub fn new() -> Self {
        Self { objects: Vec::new() }
    }

    pub fn add(&mut self, object: Arc<dyn Hittable>) {
        self.objects.push(object);
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }
}

impl Hittable for HittableList {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut closest_so_far = t_max;
        let mut hit_record = None;

        for object in &self.objects {
            if let Some(temp_rec) = object.hit(ray, t_min, closest_so_far) {
                closest_so_far = temp_rec.t;
                hit_record = Some(temp_rec);
            }
        }

        hit_record
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {

        if self.objects.is_empty() {
            return None;
        }

        let mut output_box: Option<Aabb> = None;

        for object in &self.objects {
            if let Some(bbox) = object.bounding_box(time0, time1) {
                output_box = Some(match output_box {
                    None => bbox,
                    Some(prev_box) => Aabb::surrounding_box(&prev_box, &bbox),
                });
            } else {
                return None;
            }
        }

        output_box
    }

    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f64 {
        let weight = 1.0 / self.objects.len() as f64;
        let mut sum = 0.0;

        for object in &self.objects {
            sum += weight * object.pdf_value(origin, direction);
        }

        sum
    }

    fn random(&self, origin: &Point3) -> Vec3 {
        let int_size = self.objects.len() as i32;
        let idx = random_int(0, int_size - 1) as usize;
        self.objects[idx].random(origin)
    }
}
