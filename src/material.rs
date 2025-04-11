use std::sync::Arc;

use crate::{hittable::HitRecord, ray::Ray, rtweekend::random_double, texture::Texture, vec3::{dot, random_in_unit_sphere, random_unit_vector, reflect, refract, unit_vector, Color, Point3, Vec3}};

pub trait Material: Send + Sync {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)>;

    fn emitted(&self, _u: f64, _v: f64, _p: &Point3) -> Color {
        Color::new(0.0, 0.0, 0.0)
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

impl Material for Isotropic {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let scattered = Ray::with_time(
            rec.p,
            random_in_unit_sphere(),
            r_in.time,
        );
        let attenuation = self.albedo.value(rec.u, rec.v, &rec.p);
        Some((attenuation, scattered))
    }
}

impl Material for Lambertian {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let mut scatter_direction = rec.normal + random_unit_vector();

        if scatter_direction.near_zero() {
            scatter_direction = rec.normal
        }

        let scattered = Ray::with_time(rec.p, scatter_direction, r_in.time());
        let attenuation = self.albedo.value(rec.u, rec.v, &rec.p);

        Some((attenuation, scattered))
    }
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let reflected = reflect(&r_in.direction.unit_vector(), &rec.normal);
        let scattered = Ray::with_time(
            rec.p,
            reflected + self.fuzz * random_in_unit_sphere(),
            r_in.time(),
        );
        let attenuation = self.albedo;

        if dot(&scattered.direction, &rec.normal) > 0.0 {
            Some((attenuation, scattered))
        } else {
            None
        }
    }
}

impl Material for Dielectric {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let attentuation: Color = Color::new(1.0, 1.0, 1.0);
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

        Some((attentuation, scattered))
    }
}

impl Material for DiffuseLight {
    fn scatter(&self, r_in: &Ray, _rec: &HitRecord) -> Option<(Color, Ray)> {
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
