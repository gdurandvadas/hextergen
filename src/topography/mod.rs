use crate::mesh::Mesh;
use crate::utils::noise::OctaveNoise;
use crate::utils::queues;
use crate::{cmd::GenerateOptions, mesh::Coord};
use hashbrown::{HashMap, HashSet};
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

pub struct Plate {
    pub direction: f32,
    pub speed: f32,
    pub area: Vec<Coord>,
    pub border: HashMap<Coord, Vec<Coord>>,
}

pub struct Plates {
    pub regions: HashMap<Coord, Plate>,
    pub map: HashMap<Coord, Coord>,
}

impl Plates {
    fn new(options: &GenerateOptions, mesh: &Mesh) -> Self {
        let mut regions = HashMap::<Coord, Plate>::new();
        let mut map = HashMap::<Coord, Coord>::new();
        let mut queue = queues::FIRO::<(Coord, Coord)>::new(options.seed);
        let mut visited = HashSet::<Coord>::new();
        let mut rng = Pcg64Mcg::seed_from_u64(options.seed);

        let seeds = Seeds::build(options, mesh);

        for seed in seeds {
            queue.enqueue((seed, seed));
            map.insert(seed, seed);
            regions.insert(
                seed,
                Plate {
                    direction: rng.gen_range(0.0..360.0),
                    speed: 0.0,
                    area: vec![seed],
                    border: HashMap::new(),
                },
            );
            visited.insert(seed);
        }

        while let Some((current, seed)) = queue.dequeue() {
            for (neighbor, _wrapping) in &mesh.get_hex(current.x, current.y).neighbors {
                if !visited.contains(neighbor) {
                    map.insert(*neighbor, seed);
                    regions.get_mut(&seed).unwrap().area.push(*neighbor);
                    visited.insert(*neighbor);
                    queue.enqueue((*neighbor, seed));
                }
            }
        }

        Self { regions, map }
    }

    fn borders(&mut self, mesh: &Mesh) {
        self.regions.par_iter_mut().for_each(|(p_coord, plate)| {
            for hex in &plate.area {
                for (neighbor, _wrapping) in &mesh.get_hex(hex.x, hex.y).neighbors {
                    if self.map[neighbor] != *p_coord {
                        plate
                            .border
                            .entry(*neighbor)
                            .or_insert(Vec::new())
                            .push(*hex);
                    }
                }
            }
        })
    }
}

pub struct Topography {
    pub elevations: Elevations,
    pub plates: Plates,
}

impl Topography {
    pub fn new(options: &GenerateOptions, mesh: &Mesh) -> Self {
        let elevations = Elevations::build(options);
        let mut plates = Plates::new(options, &mesh);
        plates.borders(&mesh);
        Topography { elevations, plates }
    }

    pub fn get_hex(&self, x: i32, y: i32) -> &f32 {
        let coord = Coord::new(x, y);
        &self.elevations[coord.to_dim()]
    }
}
