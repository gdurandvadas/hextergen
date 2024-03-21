use image::Rgba;

pub trait Colors {
    fn values(&self) -> [u8; 3];
    fn alpha(&self, alpha: u8) -> Rgba<u8> {
        let values = self.values();
        Rgba([values[0], values[1], values[2], alpha])
    }
    fn rgba(&self) -> Rgba<u8>;
}

#[allow(dead_code)]
pub enum Debug {
    Brown,
    Green,
    Red,
    Blue,
}

impl Colors for Debug {
    fn values(&self) -> [u8; 3] {
        match self {
            Self::Brown => [255, 181, 112],
            Self::Green => [97, 255, 100],
            Self::Red => [255, 112, 122],
            Self::Blue => [82, 163, 255],
        }
    }

    fn rgba(&self) -> Rgba<u8> {
        let values = self.values();
        match self {
            Self::Brown => Rgba([values[0], values[1], values[2], 255]),
            Self::Green => Rgba([values[0], values[1], values[2], 255]),
            Self::Red => Rgba([values[0], values[1], values[2], 255]),
            Self::Blue => Rgba([values[0], values[1], values[2], 255]),
        }
    }
}

impl Debug {
    pub fn from_elevation(elevation: &f32) -> Rgba<u8> {
        match elevation > &0.0 {
            true => {
                // If the elevation is greater than 0, we return a brown color.
                // The alpha value is proportional to the elevation, so higher elevations
                // result in a more opaque color.
                let alpha = ((elevation * 0.5 + 0.5) * 255.0) as u8;
                Debug::Brown.alpha(alpha)
            }
            false => {
                // If the elevation is less than or equal to 0, we return a blue color.
                // The alpha value is inversely proportional to the elevation, so lower elevations
                // (more negative) result in a more opaque color.
                let alpha = 255u8 - ((-elevation * 0.5 + 0.5) * 255.0) as u8;
                Debug::Blue.alpha(alpha)
            }
        }
    }
}
