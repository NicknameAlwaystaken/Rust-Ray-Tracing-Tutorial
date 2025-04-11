use crate::vec3::Vec3;


pub struct Onb {
    axis: [Vec3; 3],
}

impl Onb {
    pub fn new() -> Self {
        Self {
            axis: [Vec3::ZERO, Vec3::ZERO, Vec3::ZERO],
        }
    }

    pub fn u(&self) -> Vec3 {
        self.axis[0]
    }

    pub fn v(&self) -> Vec3 {
        self.axis[1]
    }

    pub fn w(&self) -> Vec3 {
        self.axis[2]
    }

    pub fn local(&self, a: f64, b: f64, c: f64) -> Vec3 {
        self.u() * a + self.v() * b + self.w() * c
    }

    pub fn local_vec(&self, a: Vec3) -> Vec3 {
        self.u() * a.x + self.v() * a.y + self.w() * a.z
    }

    pub fn build_from_w(n: Vec3) -> Self {
        let w = n.unit_vector();
        let a = if w.x.abs() > 0.9 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        let v = (w.cross(&a)).unit_vector();
        let u = v.cross(&w);

        Self {
            axis: [u, v, w,],
        }
    }
}
