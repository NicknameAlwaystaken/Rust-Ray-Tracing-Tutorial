use std::sync::Arc;

use rand::random;

use crate::{hittable::Hittable, onb::Onb, rtweekend::{random_double, PI}, vec3::{random_cosine_direction, Point3, Vec3}};



pub trait Pdf: Send + Sync {
    fn value(&self, direction: &Vec3) -> f64;
    fn generate(&self) -> Vec3;
}

pub struct CosinePdf {
    uvw: Onb,
}

pub struct HittablePdf {
    origin: Point3,
    ptr: Arc<dyn Hittable>,
}

pub struct MixturePdf {
    p: [Arc<dyn Pdf>; 2],
}

impl MixturePdf {
    pub fn new(p0: Arc<dyn Pdf>, p1: Arc<dyn Pdf>) -> Self {
        Self { p: [p0, p1] }
    }
}

impl HittablePdf {
    pub fn new(ptr: Arc<dyn Hittable>, origin: Point3) -> Self {
        Self {
            origin,
            ptr,
        }
    }
}

impl CosinePdf {
    pub fn new(w: Vec3) -> Self {
        Self {
            uvw: Onb::build_from_w(w),
        }
    }
}

impl Pdf for MixturePdf {
    fn value(&self, direction: &Vec3) -> f64 {
        0.5 * self.p[0].value(direction) + 0.5 * self.p[1].value(direction)
    }

    fn generate(&self) -> Vec3 {
        if random_double() < 0.5 {
            self.p[0].generate()
        } else {
            self.p[1].generate()
        }
    }
}

impl Pdf for HittablePdf {
    fn value(&self, direction: &Vec3) -> f64 {
        self.ptr.pdf_value(&self.origin, direction)
    }

    fn generate(&self) -> Vec3 {
        self.ptr.random(&self.origin)
    }
}

impl Pdf for CosinePdf {
    fn value(&self, direction: &Vec3) -> f64 {
       let cosine = direction.unit_vector().dot(&self.uvw.w()) ;
        if cosine <= 0.0 {
            0.0
        } else {
            cosine / PI
        }
    }

    fn generate(&self) -> Vec3 {
        self.uvw.local_vec(random_cosine_direction())
    }
}
