use std::sync::Arc;

use crate::{hittable::HitRecord, onb::Onb, ray::Ray, rtweekend::{random_double, PI}, texture::{SolidColor, Texture}, vec3::{dot, random_cosine_direction, random_in_hemisphere, random_in_unit_sphere, random_unit_vector, reflect, refract, unit_vector, Color, Point3, Vec3}};

pub trait Material: Send + Sync {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray, f64)>;

    fn emitted(&self, _u: f64, _v: f64, _p: &Point3) -> Color {
        Color::new(0.0, 0.0, 0.0)
    }

    fn scattering_pdf(&self, r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        0.0
    }
}

pub struct Lambertian {
    pub albedo: Arc<dyn Texture>,
}

pub struct Metal {
    pub albedo: Color,
    pub fuzz: f64,
}

pub struct Dielectric {
    pub ir: f64,
}

pub struct DiffuseLight {
    pub emit: Arc<dyn Texture>,
}

pub struct Isotropic {
    pub albedo: Arc<dyn Texture>,
}

impl Isotropic {
    pub fn new( albedo: Arc<dyn Texture>) -> Self {
        Self {
            albedo,
        }
    }
}

impl DiffuseLight {
    pub fn new( emit: Arc<dyn Texture>) -> Self {
        Self {
            emit,
        }
    }
}

impl Lambertian {
    pub fn new_from_texture( albedo: Arc<dyn Texture>) -> Self {
        Self {
            albedo,
        }
    }

    pub fn new_from_color( c: Color) -> Self {
        Self {
            albedo: Arc::new(SolidColor::new(c)),
        }
    }
}

impl Dielectric {
    pub fn new( ir: f64) -> Self {
        Self {
            ir,
        }
    }
}

impl Metal {
    pub fn new( albedo: Color, fuzz: f64) -> Self {
        Self {
            albedo,
            fuzz,
        }
    }
}

impl Material for Isotropic {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray, f64)> {
        let scattered = Ray::with_time(
            rec.p,
            random_in_unit_sphere(),
            r_in.time,
        );
        let attenuation = self.albedo.value(rec.u, rec.v, &rec.p);
        let pdf = 1.0 / (4.0 * PI);
        Some((attenuation, scattered, pdf))
    }
}

impl Material for Lambertian {
    /*
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray, f64)> {
        let mut scatter_direction = rec.normal + random_unit_vector();

        if scatter_direction.near_zero() {
            scatter_direction = rec.normal
        }

        let scattered = Ray::with_time(rec.p, scatter_direction, r_in.time());
        let attenuation = self.albedo.value(rec.u, rec.v, &rec.p);
        let cosine = dot(&rec.normal, &scattered.direction.unit_vector());
        let pdf = if cosine < 0.0 { 0.0 } else { cosine / PI };

        Some((attenuation, scattered, pdf))
    }
    */

    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray, f64)> {
        let uvw = Onb::build_from_w(rec.normal);
        let direction = uvw.local_vec(random_cosine_direction());
        let scattered = Ray::with_time(rec.p, direction.unit_vector(), r_in.time());
        let attenuation = self.albedo.value(rec.u, rec.v, &rec.p);
        let pdf = uvw.w().dot(&scattered.direction.unit_vector()) / PI;

        Some((attenuation, scattered, pdf))
    }

    fn scattering_pdf(&self, r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        let cosine = dot(&rec.normal, &scattered.direction.unit_vector());
        if cosine < 0.0 {
            0.0
        } else {
            cosine / PI
        }
    }

}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray, f64)> {
        let reflected = reflect(&r_in.direction.unit_vector(), &rec.normal);
        let scattered = Ray::with_time(
            rec.p,
            reflected + self.fuzz * random_in_unit_sphere(),
            r_in.time(),
        );
        let attenuation = self.albedo;

        if dot(&scattered.direction, &rec.normal) > 0.0 {
            Some((attenuation, scattered, 1.0))
        } else {
            None
        }
    }
}

impl Material for Dielectric {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray, f64)> {
        let attenuation: Color = Color::new(1.0, 1.0, 1.0);
        let refraction_ratio: f64 = if rec.front_face {
            1.0 / self.ir
        } else {
            self.ir
        };

        let unit_direction: Vec3 = r_in.direction.unit_vector();
        let cos_theta = (dot(&-unit_direction, &rec.normal)).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        let direction: Vec3 = if cannot_refract || reflectance(cos_theta, refraction_ratio) > random_double() {
            reflect(&unit_direction, &rec.normal)
        } else {
            refract(&unit_direction, &rec.normal, refraction_ratio)
        };

        let scattered = Ray::with_time(rec.p, direction, r_in.time());

        Some((attenuation, scattered, 1.0))
    }

    fn scattering_pdf(&self, r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        1.0
    }
}

impl Material for DiffuseLight {
    fn scatter(&self, r_in: &Ray, _rec: &HitRecord) -> Option<(Color, Ray, f64)> {
        None
    }

    fn emitted(&self, u: f64, v: f64, p: &Point3) -> Color {
        self.emit.value(u, v, p)
    }
}

fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
    // Use Schlick's approximation for reflectance.
    let mut r0 = (1.0-ref_idx) / (1.0+ref_idx);
    r0 = r0*r0;
    r0 + (1.0-r0)*(1.0-cosine).powf(5.0)
}
