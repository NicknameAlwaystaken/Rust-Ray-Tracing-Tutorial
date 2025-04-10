use crate::{rtweekend::{random_double, random_int}, vec3::Point3};


pub struct Perlin {
    ranfloat: Vec<f64>,
    perm_x: Vec<usize>,
    perm_y: Vec<usize>,
    perm_z: Vec<usize>,
}


impl Perlin {
    const POINT_COUNT: usize = 256;

    pub fn new() -> Self {
        let ranfloat = (0..Self::POINT_COUNT)
            .map(|_| random_double())
            .collect::<Vec<_>>();

        let perm_x = Self::perlin_generate_perm();
        let perm_y = Self::perlin_generate_perm();
        let perm_z = Self::perlin_generate_perm();

        Self {
            ranfloat,
            perm_x,
            perm_y,
            perm_z
        }
    }

    pub fn noise(&self, p: &Point3) -> f64 {
        let i = (4.0 * p.x) as i32 & 255;
        let j = (4.0 * p.y) as i32 & 255;
        let k = (4.0 * p.z) as i32 & 255;

        let idx = self.perm_x[i as usize] ^ self.perm_y[j as usize] ^ self.perm_z[k as usize];
        self.ranfloat[idx]
    }

    fn perlin_generate_perm() -> Vec<usize> {
        let mut p: Vec<usize> = (0..Self::POINT_COUNT).collect();
        Self::permute(&mut p);
        p
    }

    fn permute(p: &mut Vec<usize>) {
        for i in (1..p.len()).rev() {
            let target = random_int(0, i as i32);
            p.swap(i, target as usize);
        }
    }
}
