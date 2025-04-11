use std::sync::Arc;

use crate::{hittable::Hittable, onb::Onb, rtweekend::PI, vec3::{random_cosine_direction, Point3, Vec3}};



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
