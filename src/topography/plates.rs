use crate::mesh::Mesh;
use crate::utils::queues;
use crate::{cmd::GenerateOptions, mesh::Coord};
use hashbrown::{HashMap, HashSet};
use log::debug;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use rayon::prelude::*;

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

const MAP_EDGE: Coord = Coord { x: -1, y: -1 };

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
    pub fn new(options: &GenerateOptions, mesh: &Mesh) -> Self {
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

    pub fn borders(&mut self, mesh: &Mesh) {
        self.regions.par_iter_mut().for_each(|(p_coord, plate)| {
            for hex in &plate.area {
                let neighbors = &mesh.get_hex(hex.x, hex.y).neighbors;
                if neighbors.len() < 6 {
                    plate
                        .border
                        .entry(MAP_EDGE)
                        .or_insert(Vec::new())
                        .push(*hex);
                } else {
                    neighbors.iter().for_each(|(neighbor, _wrapping)| {
                        let n_p_coord = self.map[neighbor];
                        if n_p_coord != *p_coord {
                            plate
                                .border
                                .entry(n_p_coord)
                                .or_insert(Vec::new())
                                .push(*hex);
                        }
                    });
                }
            }
        })
    }
}
