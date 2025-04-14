use std::sync::Arc;

use crate::{hittable::HitRecord, onb::Onb, pdf::{CosinePdf, Pdf}, ray::Ray, rtweekend::{random_double, PI}, texture::{SolidColor, Texture}, vec3::{dot, random_cosine_direction, random_in_hemisphere, random_in_unit_sphere, random_unit_vector, reflect, refract, unit_vector, Color, Point3, Vec3}};

pub trait Material: Send + Sync {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        None
    }

    fn emitted(&self, _u: f64, _v: f64, _p: &Point3, rec: &HitRecord) -> Color {
        Color::new(0.0, 0.0, 0.0)
    }

    fn scattering_pdf(&self, r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        0.0
    }
}

pub struct ScatterRecord {
    pub attenuation: Color,
    pub pdf_ptr: Option<Arc< dyn Pdf>>,
    pub skip_pdf: bool,
    pub skip_pdf_ray: Ray,
}

pub struct EmptyMaterial;

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

impl Material for EmptyMaterial {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        None
    }

    fn emitted(&self, _u: f64, _v: f64, _p: &Point3, rec: &HitRecord) -> Color {
        Color::new(0.0, 0.0, 0.0)
    }

    fn scattering_pdf(&self, r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        0.0
    }
}

impl Material for Isotropic {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        Some(ScatterRecord {
                attenuation: self.albedo.value(rec.u, rec.v, &rec.p),
                pdf_ptr: Some(Arc::new(CosinePdf::new(rec.normal))),
                skip_pdf: false,
                skip_pdf_ray: Ray::default(),
            })
    }

    fn scattering_pdf(&self, r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        1.0 / (4.0 * PI)
    }
}

impl Material for Lambertian {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {

        Some(ScatterRecord {
                attenuation: self.albedo.value(rec.u, rec.v, &rec.p),
                pdf_ptr: Some(Arc::new(CosinePdf::new(rec.normal))),
                skip_pdf: false,
                skip_pdf_ray: Ray::default(),
            })
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
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let reflected = reflect(&r_in.direction.unit_vector(), &rec.normal)
                        + self.fuzz * random_in_unit_sphere();

        Some(ScatterRecord {
                attenuation: self.albedo,
                pdf_ptr: None,
                skip_pdf: true,
                skip_pdf_ray: Ray::with_time(rec.p, reflected, r_in.time()),
            })
    }
}

impl Material for Dielectric {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
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

        Some(ScatterRecord {
                attenuation,
                pdf_ptr: None,
                skip_pdf: true,
                skip_pdf_ray: Ray::with_time(rec.p, direction, r_in.time()),
            })
    }

    fn scattering_pdf(&self, r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        1.0
    }
}

impl Material for DiffuseLight {
    fn scatter(&self, r_in: &Ray, _rec: &HitRecord) -> Option<ScatterRecord> {
        None
    }

    fn emitted(&self, u: f64, v: f64, p: &Point3, rec: &HitRecord) -> Color {
        if rec.front_face {
            self.emit.value(u, v, p)
        } else {
            Color::new(0.0, 0.0, 0.0)
        }
    }
}

fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
    // Use Schlick's approximation for reflectance.
    let mut r0 = (1.0-ref_idx) / (1.0+ref_idx);
    r0 = r0*r0;
    r0 + (1.0-r0)*(1.0-cosine).powf(5.0)
}
