use hexx::{Hex as Hexx, HexLayout, OffsetHexMode, Vec2};
use log::error;
use ndarray::{Array2, Dim};
use rayon::prelude::*;

use crate::cmd::GenerateOptions;

type WrapAround = bool;

struct Coord {
    x: i32,
    y: i32,
}

impl Coord {
    fn new(x: i32, y: i32) -> Coord {
        Coord { x, y }
    }

    fn from_u32(x: u32, y: u32) -> Coord {
        Coord {
            x: x as i32,
            y: y as i32,
        }
    }

    fn from_array(arr: [i32; 2]) -> Coord {
        Coord {
            x: arr[0],
            y: arr[1],
        }
    }

    fn to_array(&self) -> [i32; 2] {
        [self.x, self.y]
    }

    fn to_dim(&self) -> Dim<[usize; 2]> {
        Dim([self.x as usize, self.y as usize])
    }

    fn wrap_x(&self, width: i32) -> Self {
        Coord {
            x: (self.x + width) % width,
            y: self.y,
        }
    }
}

struct Hex {
    axial: Hexx,
    offset: Coord,
    center: Vec2,
    corners: [Vec2; 6],
    neighbors: Vec<(Coord, WrapAround)>,
}

impl Hex {
    fn new(
        offset: Coord,
        layout: &HexLayout,
        offset_mode: OffsetHexMode,
        map_width: u32,
        map_height: u32,
    ) -> Self {
        let axial = Hexx::from_offset_coordinates(offset.to_array(), offset_mode);
        let center = layout.hex_to_world_pos(axial);
        let corners = layout.hex_corners(axial);
        let neighbors = axial
            .all_neighbors()
            .iter()
            .filter_map(|&n_axial| {
                let n_offset = n_axial.to_offset_coordinates(offset_mode);
                let n_coord = Coord::from_array(n_offset);
                if n_coord.y >= 0 && n_coord.y < map_height as i32 {
                    if n_coord.x < 0 || n_coord.x >= map_width as i32 {
                        Some((n_coord.wrap_x(map_width as i32), true))
                    } else {
                        Some((n_coord, false))
                    }
                } else {
                    None
                }
            })
            .collect();

        Self {
            axial,
            offset,
            center,
            corners,
            neighbors,
        }
    }
}

type Hexes = Array2<Hex>;

trait HexesBuilder {
    fn new_hexes(
        map_width: u32,
        map_height: u32,
        layout: &HexLayout,
        offset_mode: OffsetHexMode,
    ) -> Hexes;
}

impl HexesBuilder for Hexes {
    fn new_hexes(
        map_width: u32,
        map_height: u32,
        layout: &HexLayout,
        offset_mode: OffsetHexMode,
    ) -> Hexes {
        let builder: Vec<Hex> = (0..map_width)
            .into_par_iter()
            .flat_map(|x| {
                (0..map_height).into_par_iter().map(move |y| {
                    let offset = Coord::from_u32(x, y);
                    Hex::new(offset, layout, offset_mode, map_width, map_height)
                })
            })
            .collect();
        let hexes = Array2::from_shape_vec((map_width as usize, map_height as usize), builder);
        match hexes {
            Ok(h) => h,
            Err(e) => {
                error!("Error creating hexes: {}", e);
                std::process::exit(1);
            }
        }
    }
}

struct Screen {
    displacement: Vec2,
    resolution: Vec2,
}

impl Screen {
    fn new(map_width: u32, map_height: u32, hexes: &Hexes) -> Self {
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        let mut update_bounds = |x: f32, y: f32| {
            if x < min_x {
                min_x = x;
            }
            if x > max_x {
                max_x = x;
            }
            if y < min_y {
                min_y = y;
            }
            if y > max_y {
                max_y = y;
            }
        };

        let process_hex = |coord: &Coord, update_bounds: &mut dyn FnMut(f32, f32)| {
            let hex = &hexes[coord.to_dim()];
            for corner in hex.corners.iter() {
                update_bounds(corner.x, corner.y);
            }
            for (n_key, wrapping) in hex.neighbors.iter().filter_map(|n| Some(n)) {
                if !wrapping {
                    let neighbor_hex = &hexes[n_key.to_dim()];
                    for corner in neighbor_hex.corners.iter() {
                        update_bounds(corner.x, corner.y);
                    }
                }
            }
        };

        // Process first hex and its neighbours
        process_hex(&Coord::new(0, 0), &mut update_bounds);

        // Process last hex and its neighbours
        process_hex(
            &Coord::new(map_width as i32 - 1, map_height as i32 - 1),
            &mut update_bounds,
        );

        Self {
            displacement: Vec2::new(min_x, min_y),
            resolution: Vec2::new(max_x, max_y),
        }
    }
}

pub struct Mesh {
    pub hexes: Hexes,
    pub screen: Screen,
}

impl Mesh {
    pub fn new(options: &GenerateOptions) -> Self {
        let layout = HexLayout {
            invert_y: true,
            orientation: options.orientation,
            ..Default::default()
        };
        let offset_mode = options.offset_mode();
        let hexes = Hexes::new_hexes(options.width, options.height, &layout, offset_mode);
        let screen = Screen::new(options.width, options.height, &hexes);

        Self { hexes, screen }
    }
}
