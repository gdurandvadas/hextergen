use crate::mesh::Mesh;
use crate::utils::noise::OctaveNoise;
use crate::{cmd::GenerateOptions, mesh::Coord};
use ndarray::Array2;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
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

type Seeds = Vec<Coord>;

trait SeedsBuilder {
    fn build(options: &GenerateOptions, mesh: &Mesh) -> Seeds;
}

impl SeedsBuilder for Seeds {
    fn build(options: &GenerateOptions, mesh: &Mesh) -> Seeds {
        let mut seeds: Seeds = Vec::new();
        let mut rng = Pcg64Mcg::seed_from_u64(options.seed);
        let diagonal = (options.width.pow(2) as f32 + options.height.pow(2) as f32).sqrt();
        let min_distance = diagonal / 75.0 * 2.5;

        while seeds.len() < 75 {
            let x = rng.gen_range(2..options.width - 2);
            let y = rng.gen_range(2..options.height - 2);
            let candidate_hex = mesh.get_hex(x as i32, y as i32);
            // if seeds.iter().all(|c: &(i32, i32)| {
            //     mesh.get_hex(c., y)
            //         .position
            //         .distance_to(candidate_hex.position) as f32
            //         > min_distance
            // }) {
            //     seeds.push(coord);
            // }
            if seeds.iter().all(|c: &Coord| {
                mesh.get_hex(c.x, c.y)
                    .axial
                    .distance_to(candidate_hex.axial) as f32
                    > min_distance
            }) {
                seeds.push(Coord::new(x as i32, y as i32));
            }
        }

        seeds
    }
}

pub struct Topography {
    pub elevations: Elevations,
    pub seeds: Seeds,
}

impl Topography {
    pub fn new(options: &GenerateOptions, mesh: &Mesh) -> Self {
        let elevations = Elevations::build(options);
        let seeds = Seeds::build(options, mesh);

        Topography { elevations, seeds }
    }

    pub fn get_hex(&self, x: i32, y: i32) -> &f32 {
        let coord = Coord::new(x, y);
        &self.elevations[coord.to_dim()]
    }
}
