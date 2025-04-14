use crate::vec3::{Vec3, Point3};

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
    pub time: f64,
}

impl Ray {
    pub fn new(origin: Point3, direction: Vec3) -> Self {
        Self { origin, direction, time: 0.0}
    }

    pub fn with_time(origin: Point3, direction: Vec3, time: f64) -> Self {
        Self { origin, direction, time}
    }

    pub fn origin(&self) -> Point3 {
        self.origin
    }


    pub fn direction(&self) -> Vec3 {
        self.direction
    }


    pub fn at(&self, t: f64) -> Point3 {
        self.origin + self.direction * t
    }

    pub fn time(&self) -> f64 {
        self.time
    }
}

impl Default for Ray {
    fn default() -> Self {
        Self {
            origin: Point3::default(),
            direction: Vec3::default(),
            time: 0.0,
        }
    }
}
