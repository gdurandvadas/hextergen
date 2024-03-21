use image::Rgba;

pub trait Colors {
    fn values(&self) -> [u8; 3];
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
            Self::Brown => [177, 132, 80],
            Self::Green => [65, 159, 75],
            Self::Red => [181, 72, 81],
            Self::Blue => [59, 105, 145],
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
