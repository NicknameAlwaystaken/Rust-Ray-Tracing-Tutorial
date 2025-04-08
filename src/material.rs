use crate::{hittable::HitRecord, ray::Ray, vec3::{dot, random_in_unit_sphere, random_unit_vector, reflect, refract, unit_vector, Color, Vec3}};

pub trait Material: Send + Sync {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)>;
}

pub struct Lambertian {
    pub albedo: Color,
}

pub struct Metal {
    pub albedo: Color,
    pub fuzz: f64,
}

pub struct Dielectric {
    pub ir: f64,
}

impl Material for Lambertian {
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let mut scatter_direction = rec.normal + random_unit_vector();

        if scatter_direction.near_zero() {
            scatter_direction = rec.normal
        }

        let scattered = Ray::new(rec.p, scatter_direction);
        let attenuation = self.albedo;

        Some((attenuation, scattered))
    }
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let reflected = reflect(&r_in.direction.unit_vector(), &rec.normal);
        let scattered = Ray::new(
            rec.p,
            reflected + self.fuzz * random_in_unit_sphere(),
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
        let direction: Vec3 = if cannot_refract {
            reflect(&unit_direction, &rec.normal)
        } else {
            refract(&unit_direction, &rec.normal, refraction_ratio)
        };

        let scattered = Ray::new(rec.p, direction);

        Some((attentuation, scattered))
    }
}
