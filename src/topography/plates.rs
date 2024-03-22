use crate::mesh::{Hex, Mesh};
use crate::utils::queues;
use crate::{cmd::GenerateOptions, mesh::Coord};
use hashbrown::{HashMap, HashSet};
use hexx::Vec2;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use rayon::prelude::*;
use std::f32::consts::PI;

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

enum Slope {
    Convergent,
    Divergent,
}

enum Angle {
    In,
    Out,
}

impl Angle {
    fn from_degree(degree: f32) -> Self {
        if degree >= 270.0 || degree <= 90.0 {
            Angle::In
        } else {
            Angle::Out
        }
    }
}

impl Slope {
    fn new(
        origin_center: &Hex,
        origin_direction: f32,
        origin_magnitude: f32,
        other_center: &Hex,
        other_direction: f32,
        other_magnitude: f32,
    ) -> Self {
        let angle_between = Self::angle_between(&origin_center.center, &other_center.center);
        let origin_angle = (origin_direction - angle_between).rem_euclid(360.0);
        let other_angle = (other_direction - angle_between).rem_euclid(360.0);
        let origin_direction = Angle::from_degree(origin_angle);
        let other_direction = Angle::from_degree(other_angle);

        use Angle::*;
        match (origin_direction, other_direction) {
            (In, In) => Slope::Convergent,
            (Out, Out) => Slope::Convergent,
            (In, Out) => {
                if origin_magnitude > other_magnitude {
                    Slope::Convergent
                } else {
                    Slope::Divergent
                }
            }
            (Out, In) => {
                if other_magnitude > origin_magnitude {
                    Slope::Convergent
                } else {
                    Slope::Divergent
                }
            }
        }
    }

    fn angle_between(origin: &Vec2, other: &Vec2) -> f32 {
        ((other.y - origin.y).atan2(other.x - origin.x) * 180.0 / PI).rem_euclid(360.0)
    }
}

pub struct Interaction {
    pub slope: Slope,
    pub segment: Vec<Coord>,
}

pub struct Plate {
    pub direction: f32,
    pub area: Vec<Coord>,
    pub border: HashMap<Coord, Interaction>,
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
        let directions: HashMap<Coord, f32> = self
            .regions
            .par_iter()
            .map(|(p_coord, plate)| (*p_coord, plate.direction))
            .collect();
        let magnitudes: HashMap<Coord, f32> = self
            .regions
            .par_iter()
            .map(|(p_coord, plate)| (*p_coord, plate.area.len() as f32))
            .collect();

        self.regions.par_iter_mut().for_each(|(p_coord, plate)| {
            for hex in &plate.area {
                let neighbors = &mesh.get_hex(hex.x, hex.y).neighbors;

                if neighbors.len() < 6 {
                    // Edge of the map

                    plate
                        .border
                        .entry(MAP_EDGE)
                        .or_insert(Interaction {
                            slope: Slope::Divergent,
                            segment: Vec::new(),
                        })
                        .segment
                        .push(*hex);
                } else {
                    // Regular interaction

                    neighbors.iter().for_each(|(neighbor, _wrapping)| {
                        let n_p_coord = self.map[neighbor];

                        if n_p_coord != *p_coord {
                            let origin_direction = directions[p_coord];
                            let other_direction = directions[&n_p_coord];
                            let origin_magnitude = magnitudes[p_coord];
                            let other_magnitude = magnitudes[&n_p_coord];

                            plate
                                .border
                                .entry(n_p_coord)
                                .or_insert({
                                    let slope = Slope::new(
                                        &mesh.get_hex(hex.x, hex.y),
                                        origin_direction,
                                        origin_magnitude,
                                        &mesh.get_hex(n_p_coord.x, n_p_coord.y),
                                        other_direction,
                                        other_magnitude,
                                    );
                                    Interaction {
                                        slope,
                                        segment: Vec::new(),
                                    }
                                })
                                .segment
                                .push(*hex);
                        }
                    });
                }
            }
        })
    }
}
