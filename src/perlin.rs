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
        let mut u = p.x - p.x.floor();
        let mut v = p.y - p.y.floor();
        let mut w = p.z - p.z.floor();
        u = u*u*(3.0 - 2.0*u);
        v = v*v*(3.0 - 2.0*v);
        w = w*w*(3.0 - 2.0*w);

        let i = p.x.floor() as i32;
        let j = p.y.floor() as i32;
        let k = p.z.floor() as i32;

        let mut c = [[[0.0f64; 2]; 2]; 2];

        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    let idx = self.perm_x[((i + di as i32) & 255) as usize]
                            ^ self.perm_y[((j + dj as i32) & 255) as usize]
                            ^ self.perm_z[((k + dk as i32) & 255) as usize];
                    c[di][dj][dk] = self.ranfloat[idx];
                }
            }
        }

        Self::trilinear_interp(&c, u, v, w)
    }

    fn trilinear_interp(c: &[[[f64; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let mut accum = 0.0;

        for i in 0..2 {
            let ui = if i == 1 { u } else { 1.0 - u };

            for j in 0..2 {
                let vj = if j == 1 { v } else { 1.0 - v };

                for k in 0..2 {
                    let wk = if k == 1 { w } else { 1.0 - w };

                    accum += ui * vj * wk * c[i][j][k];
                }
            }
        }

        accum
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
