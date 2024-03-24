mod plates;

use log::debug;
pub use plates::InteractionVariant;
use crate::mesh::Mesh;
use crate::utils::noise::OctaveNoise;
use crate::{cmd::GenerateOptions, mesh::Coord};
use ndarray::Array2;
use plates::Plates;
use rayon::prelude::*;

type Elevations = Array2<f32>;

trait ElevationsBuilder {
    fn build(options: &GenerateOptions) -> Elevations;
}

impl ElevationsBuilder for Elevations {
    fn build(options: &GenerateOptions) -> Elevations {
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

        Array2::from_shape_vec((options.width as usize, options.height as usize), builder)
            .expect("Error creating hexes: failed to match grid dimensions with hex count.")
    }
}

pub struct Topography {
    pub elevations: Elevations,
    pub plates: Plates,
}

impl Topography {
    pub fn new(options: &GenerateOptions, mesh: &Mesh) -> Self {
        let mut elevations = Elevations::build(options);
        let mut plates = Plates::new(options, &mesh);
        plates.borders(&mesh);
        plates.slopes(&mesh);

        plates.regions.iter().for_each(|(_p_coord, plate)| {
            plate.slopes.iter().for_each(|slope| {
                slope.hexes.iter().enumerate().for_each(|(i, hex)| {
                    elevations
                        .get_mut([hex.x as usize, hex.y as usize])
                        .map(|elevation| {
                            *elevation =
                                slope
                                    .variant
                                    .effect(i, slope.hexes.len(), elevation.clone())
                        });
                })
            })
        });

        let min_elevation =
            elevations.iter().fold(
                f32::MAX,
                |acc, &elevation| {
                    if elevation < acc {
                        elevation
                    } else {
                        acc
                    }
                },
            );
        let max_elevation =
            elevations.iter().fold(
                f32::MIN,
                |acc, &elevation| {
                    if elevation > acc {
                        elevation
                    } else {
                        acc
                    }
                },
            );

        debug!("Min elevation: {}", min_elevation);
        debug!("Max elevation: {}", max_elevation);

        Topography { elevations, plates }
    }

    pub fn get_hex(&self, x: i32, y: i32) -> &f32 {
        let coord = Coord::new(x, y);
        &self.elevations[coord.to_dim()]
    }
}
