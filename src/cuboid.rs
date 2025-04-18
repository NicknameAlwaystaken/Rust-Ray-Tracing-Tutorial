use std::sync::Arc;

use crate::{aabb::Aabb, aarect::{XYRect, XZRect, YZRect}, hittable::Hittable, hittable_list::HittableList, material::Material, vec3::Point3};


pub struct Cuboid {
    pub box_min: Point3,
    pub box_max: Point3,
    pub sides: HittableList,
}

impl Cuboid {
    pub fn new(p0: Point3, p1: Point3, material: Arc<dyn Material>) -> Self {
        let mut sides = HittableList::new();

        sides.add(Arc::new(XYRect::new(p0.x, p1.x, p0.y, p1.y, p1.z, Arc::clone(&material))));
        sides.add(Arc::new(XYRect::new(p0.x, p1.x, p0.y, p1.y, p0.z, Arc::clone(&material))));

        sides.add(Arc::new(XZRect::new(p0.x, p1.x, p0.z, p1.z, p1.y, Arc::clone(&material))));
        sides.add(Arc::new(XZRect::new(p0.x, p1.x, p0.z, p1.z, p0.y, Arc::clone(&material))));

        sides.add(Arc::new(YZRect::new(p0.y, p1.y, p0.z, p1.z, p1.x, Arc::clone(&material))));
        sides.add(Arc::new(YZRect::new(p0.y, p1.y, p0.z, p1.z, p0.x, Arc::clone(&material))));

        Self {
            box_min: p0,
            box_max: p1,
            sides
        }
    }
}

impl Hittable for Cuboid {
    fn hit(&self, r: &crate::ray::Ray, t_min: f64, t_max: f64) -> Option<crate::hittable::HitRecord> {
       self.sides.hit(r, t_min, t_max)
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<crate::aabb::Aabb> {
       Some(Aabb::new(self.box_min, self.box_max))
    }
}
