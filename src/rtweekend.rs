use rand::prelude::*;

use crate::vec3::Point3;

pub const INFINITY: f64 = f64::INFINITY;
pub const PI: f64 = 3.1415926535897932385;

pub fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * PI / 180.0
}

pub fn random_double() -> f64 {
    let mut rng = rand::rng();
    rng.random::<f64>()
}

pub fn random_double_range(min: f64, max:f64) -> f64 {
    let mut rng = rand::rng();
    rng.gen_range(min..max)
}

pub fn clamp(x: f64, min: f64, max: f64) -> f64 {
    if x < min { return min; }
    if x > max { return max; }
    x
}

pub fn random_int(min: i32, max: i32) -> i32 {
    (random_double_range(min as f64, (max + 1) as f64)) as i32
}

pub fn get_sphere_uv(p: &Point3) -> (f64, f64) {
    // p: a given point on the sphere of radius one, centered at the origin.
    // u: returned value [0,1] of angle around the Y axis from X=-1.
    // v: returned value [0,1] of angle from Y=-1 to Y=+1.
    // <1 0 0> yields <0.50 0.50> <-1 0 0> yields <0.00 0.50>
    // <0 1 0> yields <0.50 1.00> < 0 -1 0> yields <0.50 0.00>
    // <0 0 1> yields <0.25 0.50> < 0 0 -1> yields <0.75 0.50>
    let theta = (-p.y).acos();
    let phi = (-p.z).atan2(p.x) + PI;
    let u = phi / (2.0 * PI);
    let v = theta / PI;
    (u, v)
}
