use std::{path::Path, sync::Arc};

use image::{DynamicImage, GenericImageView, ImageReader, Pixel};

use crate::{perlin::Perlin, vec3::{Color, Point3}};


pub trait Texture: Send + Sync {
    fn value(&self, u: f64, v: f64, p: &Point3) -> Color;
}

pub struct SolidColor {
    color_value: Color,
}

pub struct CheckerTexture {
    pub even: Arc<dyn Texture>,
    pub odd: Arc<dyn Texture>,
}

pub struct NoiseTexture {
    noise: Perlin,
    scale: f64,
}

pub struct ImageTexture {
    image: Option<DynamicImage>,
    width: usize,
    height: usize,
}

impl NoiseTexture {
    pub fn new(scale: f64) -> Self {
        Self {
            noise: Perlin::new(),
            scale,
        }
    }
}

impl ImageTexture {
    pub fn new(filename: &str) -> Self {
        let path = Path::new(filename);

        match ImageReader::open(path) {
            Ok(reader) => match reader.decode() {
                Ok(img) => {
                    let (width, height) = img.dimensions();
                    Self {
                        image: Some(img),
                        width: width as usize,
                        height: height as usize,
                    }
                }
                Err(e) => {
                    eprintln!("ERROR: Could not decode image file '{}': {}", filename, e);
                    Self {
                        image: None,
                        width: 0,
                        height: 0,
                    }
                }
            },
            Err(e) => {
                eprintln!("ERROR: Could not open image file '{}': {}", filename, e);
                Self {
                    image: None,
                    width: 0,
                    height: 0,
                }
            }
        }
    }
}

impl CheckerTexture {
    pub fn new(even: Arc<dyn Texture>, odd: Arc<dyn Texture>) -> Self {
        Self { even, odd }
    }

    pub fn from_colors(c1: Color, c2: Color) -> Self {
        Self {
            even: Arc::new(SolidColor::new(c1)),
            odd: Arc::new(SolidColor::new(c2)),
        }
    }
}

impl SolidColor {
    pub fn new(color: Color) -> Self {
        Self { color_value: color }
    }

    pub fn from_rgb (r: f64, g: f64, b: f64) -> Self {
        Self {
            color_value: Color::new(r, g, b),
        }
    }
}

impl Texture for CheckerTexture {
    fn value(&self, u: f64, v: f64, p: &Point3) -> Color {
        let sines = (10.0 * p.x).sin() * (10.0 * p.y).sin() * (10.0 * p.z).sin();
        if sines < 0.0 {
            self.odd.value(u, v, p)
        } else {
            self.even.value(u, v, p)
        }
    }
}

impl Texture for SolidColor {
    fn value(&self, u: f64, v: f64, p: &Point3) -> Color {
        self.color_value
    }
}

impl Texture for NoiseTexture {
    fn value(&self, u: f64, v: f64, p: &Point3) -> Color {
        Color::new(1.0, 1.0, 1.0) * 0.5 *(1.0 + ( self.scale*p.z + 10.0 * self.noise.turb(p)).sin())
    }
}

impl Texture for ImageTexture {
    fn value(&self, mut u: f64, mut v: f64, _p: &Point3) -> Color {
        // If we have no texture data, then return solid cyan as a debugging aid.
        let img = match &self.image {
            Some(img) => img,
            None => return Color::new(0.0, 1.0, 1.0),
        };

        // Clamp input texture coordinates to [0,1] x [1,0]
        u = u.clamp(0.0, 1.0);
        v = 1.0 - v.clamp(0.0, 1.0);

        let width = self.width;
        let height = self.height;

        let mut i = (u * width as f64) as u32;
        let mut j = (v * height as f64) as u32;

        // Clamp integer mapping, since actual coordinates should be less than 1.0
        if i >= width as u32 { i = width as u32 - 1; }
        if j >= height as u32 { j = height as u32 - 1; }

        let pixel = img.get_pixel(i, j).to_rgb();
        let [r, g, b] = pixel.0;

        let color_scale = 1.0 / 255.0;
        Color::new(
            r as f64 * color_scale,
            g as f64 * color_scale,
            b as f64 * color_scale,
        )
    }
}
