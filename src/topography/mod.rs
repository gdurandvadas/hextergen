use crate::utils::noise::OctaveNoise;
use crate::{cmd::GenerateOptions, mesh::Coord};
use ndarray::Array2;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

pub struct Topography {
    pub elevations: Array2<f32>,
}

impl Topography {
    pub fn new(options: &GenerateOptions) -> Self {
        let builder: Vec<f32> = (0..options.width as i32)
            .into_par_iter()
            .flat_map(|x| {
                (0..options.height as i32).into_par_iter().map(move |y| {
                    let coord = Coord::new(x, y);
                    let noise = OctaveNoise::new(options.seed, 5.0, 1.0, 20, 0.6);

                    noise.d3(coord.to_cilinder(options.width as i32, options.height as i32))
                })
            })
            .collect();

        let elevations =
            Array2::from_shape_vec((options.width as usize, options.height as usize), builder)
                .expect("Error creating hexes: failed to match grid dimensions with hex count.");

        Topography { elevations }
    }

    pub fn get_hex(&self, x: i32, y: i32) -> &f32 {
        let coord = Coord::new(x, y);
        &self.elevations[coord.to_dim()]
    }
}
