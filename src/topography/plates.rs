use crate::{
    cmd::GenerateOptions,
    mesh::{Coord, Hex, Mesh},
    utils::queues,
};
use hashbrown::{HashMap, HashSet};
use hexx::Vec2;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use rayon::prelude::*;
use std::collections::VecDeque;
use std::f32::consts::PI;

// Seeds for the tectonic plates
type Seeds = Vec<Coord>;

trait SeedsBuilder {
    fn build(options: &GenerateOptions, mesh: &Mesh) -> Seeds;
}

impl SeedsBuilder for Seeds {
    // Generate the seeds for the tectonic plates randomly picking points in the map
    // The points must be placed at a minimum distance from each other
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

// Special seed to identify the edge of the map
const MAP_EDGE: Coord = Coord { x: -1, y: -1 };

// Angle that identifies the direction of the movements
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

// Categorize the angle relationship between two plates
#[derive(Debug, Clone, Copy)]
pub enum InteractionVariant {
    Convergent,
    Divergent,
}

impl InteractionVariant {
    // Categorize the interaction between two plates
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
            (In, In) => InteractionVariant::Convergent,
            (Out, Out) => InteractionVariant::Convergent,
            (In, Out) => {
                if origin_magnitude > other_magnitude {
                    InteractionVariant::Convergent
                } else {
                    InteractionVariant::Divergent
                }
            }
            (Out, In) => {
                if other_magnitude > origin_magnitude {
                    InteractionVariant::Convergent
                } else {
                    InteractionVariant::Divergent
                }
            }
        }
    }

    // Calculate the angle between two points
    fn angle_between(origin: &Vec2, other: &Vec2) -> f32 {
        ((other.y - origin.y).atan2(other.x - origin.x) * 180.0 / PI).rem_euclid(360.0)
    }

    pub fn effect(&self, index: usize, slope_len: usize, elevation: f32) -> f32 {
        let contrast = 1.065;
        let steepness = 0.013;

        if (index as f32) < (slope_len as f32) * 0.3 {
            return elevation * 0.995;
        }


        let i = index as f32;
        let n = (slope_len - 1) as f32;

        let distance_effect = match self {
            InteractionVariant::Convergent => i / n,
            InteractionVariant::Divergent => -i / n,
        };

        let adjustment = distance_effect * steepness;

        // Adjust the elevation by the calculated adjustment, without halving the scale
        (elevation + adjustment) * contrast
    }
}

// Represents a slope as a list of hexes between the border hex and the seed hex
#[derive(Debug)]
pub struct Slope {
    pub variant: InteractionVariant,
    pub hexes: Vec<Coord>,
}

pub type Slopes = Vec<Slope>;

trait SlopesBuilder {
    fn build(seed: &Coord, interaction: &Interaction, mesh: &Mesh) -> Slopes;
    fn wrap_distance(origin: &Vec2, other: &Vec2, resolution_x: f32) -> f32;
}

impl SlopesBuilder for Slopes {
    // Generate the slopes between the border hex and the seed hex
    // It uses an A* algorithm to find the shortest path between the two points
    // The path candidates are picked randomly for a more natural look
    fn build(seed: &Coord, interaction: &Interaction, mesh: &Mesh) -> Slopes {
        interaction
            .segment
            .par_iter()
            .map(|b_coord| {
                let b_hex = mesh.get_hex(b_coord.x, b_coord.y);
                let (x, y) = (seed.x * b_coord.x, seed.y * b_coord.y);
                let unique_seed = ((x as u64) << 32) | (y as u64);
                let mut rng = Pcg64Mcg::seed_from_u64(unique_seed);
                let mut queue = VecDeque::<Coord>::new();
                let mut visited = HashSet::<Coord>::new();
                let mut hexes = Vec::<Coord>::new();

                hexes.push(*seed);
                queue.push_back(*seed);
                visited.insert(*seed);

                while let Some(current) = queue.pop_front() {
                    let current_hex = mesh.get_hex(current.x, current.y);
                    let current_to_border = Self::wrap_distance(
                        &current_hex.center,
                        &b_hex.center,
                        mesh.screen.resolution.x,
                    );

                    let mut neighbors = current_hex
                        .neighbors
                        .iter()
                        .filter(|(n_coord, _)| !visited.contains(n_coord))
                        .collect::<Vec<_>>();

                    if neighbors.iter().any(|(n_coord, _)| n_coord == b_coord) {
                        hexes.push(*b_coord);
                        break;
                    }

                    neighbors.shuffle(&mut rng);

                    for (n_coord, _) in neighbors {
                        let neighbor_hex = mesh.get_hex(n_coord.x, n_coord.y);
                        let neighbor_to_border = Self::wrap_distance(
                            &neighbor_hex.center,
                            &b_hex.center,
                            mesh.screen.resolution.x,
                        );
                        if neighbor_to_border <= current_to_border {
                            hexes.push(*n_coord);
                            queue.push_back(*n_coord);
                            visited.insert(*n_coord);
                            break;
                        }
                    }
                }

                Slope {
                    variant: interaction.variant,
                    hexes,
                }
            })
            .collect()
    }

    // Calculate the distance between two points considering the wrapping of the map
    fn wrap_distance(origin: &Vec2, other: &Vec2, resolution_x: f32) -> f32 {
        let direct_distance = origin.distance(*other);
        let wrap_distance_x =
            (resolution_x - (origin.x - other.x).abs()).min((origin.x - other.x).abs());
        let wrap_distance_y =
            (resolution_x - (origin.y - other.y).abs()).min((origin.y - other.y).abs());

        direct_distance.min((wrap_distance_x.powi(2) + wrap_distance_y.powi(2)).sqrt())
    }
}

// Contains the interaction between self plate and others
// There is a segment of hexes that represent the border between the plates
// The variant is applied to the whole segment
#[derive(Debug)]
pub struct Interaction {
    pub variant: InteractionVariant,
    pub segment: Vec<Coord>,
}

// Represents a tectonic plate
#[derive(Debug)]
pub struct Plate {
    pub direction: f32,
    pub area: Vec<Coord>,
    pub border: HashMap<Coord, Interaction>,
    pub slopes: Slopes,
}

// Represents the tectonic plates
#[derive(Debug)]
pub struct Plates {
    pub regions: HashMap<Coord, Plate>,
    pub map: HashMap<Coord, Coord>,
}

impl Plates {
    // Generate the tectonic plates
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
                    slopes: Vec::new(),
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

    // Identify the borders between the tectonic plates
    // Each hex in the plate's area is checked for its neighbors, if any of them
    // belongs to another plate, a border is created
    // It uses the map to check which plate the neighbor belongs to
    // It identifies the direction of the movements between the plates
    // And categorizes the movements into an interaction
    // TODO: After writhing the responsabilities of this code, it's clear that needs some refactor to split responsibilities
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
                            variant: InteractionVariant::Divergent,
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
                                    let slope = InteractionVariant::new(
                                        &mesh.get_hex(hex.x, hex.y),
                                        origin_direction,
                                        origin_magnitude,
                                        &mesh.get_hex(n_p_coord.x, n_p_coord.y),
                                        other_direction,
                                        other_magnitude,
                                    );
                                    Interaction {
                                        variant: slope,
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

    // Generate the slopes between the border hex and the seed hex
    pub fn slopes(&mut self, mesh: &Mesh) {
        self.regions.par_iter_mut().for_each(|(p_coord, plate)| {
            plate.slopes = plate
                .border
                .par_iter()
                .flat_map(|(_n_coord, interaction)| Slopes::build(p_coord, interaction, mesh))
                .collect();
        });
    }
}
