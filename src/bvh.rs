use std::{cmp::Ordering, sync::Arc};

use rand::Rng;

use crate::{aabb::Aabb, hittable::{HitRecord, Hittable}, rtweekend::random_int};


pub struct BvhNode {
    left: Arc<dyn Hittable>,
    right: Arc<dyn Hittable>,
    bbox: Aabb,
}

impl BvhNode {
    pub fn new(
        src_objects: &mut [Arc<dyn Hittable>],
        time0: f64,
        time1: f64
    ) -> Self {
        let axis = random_int(0, 2);
        let comparator = match axis {
            0 => box_compare_x,
            1 => box_compare_y,
            _ => box_compare_z,
        };

        let object_span = src_objects.len();

        let (left, right): (Arc<dyn Hittable>, Arc<dyn Hittable>) = match object_span {
            1 => {
                let object = Arc::clone(&src_objects[0]);
                (object.clone(), object)
            }
            2 => {
                let (a, b) = (&src_objects[0], &src_objects[1]);
                if comparator(a, b) == Ordering::Less {
                    (Arc::clone(a), Arc::clone(b))
                } else {
                    (Arc::clone(b), Arc::clone(a))
                }
            }
            _ => {
                src_objects.sort_by(|a, b| comparator(a, b));
                let mid = object_span / 2;
                let left: Arc<dyn Hittable> = Arc::new(BvhNode::new(&mut src_objects[..mid], time0, time1));
                let right: Arc<dyn Hittable>  = Arc::new(BvhNode::new(&mut src_objects[mid..], time0, time1));
                (left, right)
            }
        };

        let box_left = left
            .bounding_box(time0, time1)
            .expect("No bounding box in BVH node (left)");

        let box_right = right
            .bounding_box(time0, time1)
            .expect("No bounding box in BVH node (right)");

        let bbox = Aabb::surrounding_box(&box_left, &box_right);

        BvhNode { left, right, bbox }
    }
}

impl Hittable for BvhNode {
    fn hit(&self, r: &crate::ray::Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
       if !self.bbox.hit(r, t_min, t_max) {
            return None;
        }

        let mut hit_record = None;
        let mut closest_so_far = t_max;

        if let Some(hit) = self.left.hit(r, t_min, closest_so_far) {
            closest_so_far = hit.t;
            hit_record = Some(hit);
        }

        if let Some(hit) = self.right.hit(r, t_min, closest_so_far) {
            hit_record = Some(hit);
        }

        hit_record
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
        Some(self.bbox)
    }
}

fn box_compare(a: &Arc<dyn Hittable>, b: &Arc<dyn Hittable>, axis: usize) -> Ordering {
    let box_a = a
        .bounding_box(0.0, 0.0)
        .expect("No bounding box in bvh_node constructor (a)");
    let box_b = b
        .bounding_box(0.0, 0.0)
        .expect("No bounding box in bvh_node constructor (b)");

    box_a.minimum[axis]
        .partial_cmp(&box_b.minimum[axis])
        .unwrap_or(Ordering::Equal)
}

pub fn box_compare_x(a: &Arc<dyn Hittable>, b: &Arc<dyn Hittable>) -> Ordering {
    box_compare(a, b, 0)
}

pub fn box_compare_y(a: &Arc<dyn Hittable>, b: &Arc<dyn Hittable>) -> Ordering {
    box_compare(a, b, 1)
}

pub fn box_compare_z(a: &Arc<dyn Hittable>, b: &Arc<dyn Hittable>) -> Ordering {
    box_compare(a, b, 2)
}
