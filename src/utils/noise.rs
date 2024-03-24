use noise::{NoiseFn, OpenSimplex};

/// The `OctaveNoise` struct represents a noise generator that uses the OpenSimplex noise function.
/// It allows you to generate noise with multiple octaves, which can create more complex and natural-looking noise patterns.
///
/// # Fields
///
/// * `noise_fn`: An instance of the OpenSimplex noise function.
/// * `frequency`: The frequency of the first octave of noise.
/// * `amplitude`: The amplitude of the first octave of noise.
/// * `octaves`: The number of octaves of noise to generate.
/// * `persistence`: The persistence of the noise. This controls how much each octave contributes to the final noise. It is typically set between 0 and 1, with higher values giving more contribution to the later octaves.
pub struct OctaveNoise {
    noise_fn: OpenSimplex,
    frequency: f32,
    amplitude: f32,
    octaves: u32,
    persistence: f32,
}

impl OctaveNoise {
    /// Creates a new `OctaveNoise` instance.
    ///
    /// # Arguments
    ///
    /// * `seed`: The seed value for the OpenSimplex noise function.
    /// * `frequency`: The frequency of the first octave of noise.
    /// * `amplitude`: The amplitude of the first octave of noise.
    /// * `octaves`: The number of octaves of noise to generate.
    /// * `persistence`: The persistence of the noise.
    ///
    /// # Returns
    ///
    /// A new `OctaveNoise` instance.
    pub fn new(seed: u64, frequency: f32, amplitude: f32, octaves: u32, persistence: f32) -> Self {
        let seed = seed as u32;
        OctaveNoise {
            noise_fn: OpenSimplex::new(seed),
            frequency,
            amplitude,
            octaves,
            persistence,
        }
    }

    /// Gets the noise value at the given coordinates.
    ///
    /// # Arguments
    ///
    /// * `x`: The x-coordinate.
    /// * `y`: The y-coordinate.
    ///
    /// # Returns
    ///
    /// The noise value at the given coordinates. This is a value between -1 and 1.
    pub fn d2(&self, x: f32, y: f32) -> f32 {
        let mut total = 0.0;
        let mut max_value = 0.0;
        let mut amplitude = self.amplitude;
        let mut frequency = self.frequency;

        for _ in 0..self.octaves {
            total += self
                .noise_fn
                .get([(x * frequency) as f64, (y * frequency) as f64]) as f32
                * amplitude;
            max_value += amplitude;
            amplitude *= self.persistence;
            frequency *= 2.0;
        }

        total / max_value
    }

    pub fn d3(&self, (x, y, z): (f32, f32, f32)) -> f32 {
        let mut total = 0.0;
        let mut max_value = 0.0;
        let mut amplitude = self.amplitude;
        let mut frequency = self.frequency;

        for _ in 0..self.octaves {
            total += self.noise_fn.get([
                (x * frequency) as f64,
                (y * frequency) as f64,
                (z * frequency) as f64,
            ]) as f32
                * amplitude;
            max_value += amplitude;
            amplitude *= self.persistence;
            frequency *= 2.0;
        }

        total / max_value
    }
}
