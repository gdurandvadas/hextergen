use crate::cmd::GenerateOptions;
use hexx::{Hex as Hexx, HexLayout, OffsetHexMode, Vec2};
use ndarray::{Array2, Dim};
use rayon::prelude::*;

type WrapAround = bool;

/// Represents a coordinate in a 2D grid.
///
/// `Coord` is used to store the position of hexagons in a hexagonal grid system.
/// It holds two integer values, `x` and `y`, representing the coordinates on the grid.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use hextergen::mesh::Coord;
///
/// let coord = Coord::new(10, 20);
/// assert_eq!(coord.x, 10);
/// assert_eq!(coord.y, 20);
/// ```
#[derive(Clone, Copy)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

impl Coord {
    /// Creates a new `Coord` with given x and y values.
    pub fn new(x: i32, y: i32) -> Coord {
        Coord { x, y }
    }

    /// Constructs a `Coord` from an array of two integers.
    fn from_array(arr: [i32; 2]) -> Coord {
        Coord {
            x: arr[0],
            y: arr[1],
        }
    }

    /// Converts `Coord` into an array of two integers.
    fn to_array(&self) -> [i32; 2] {
        [self.x, self.y]
    }

    /// Converts `Coord` to a `Dim<[usize; 2]>` for indexing into `ndarray` structures.
    fn to_dim(&self) -> Dim<[usize; 2]> {
        Dim([self.x as usize, self.y as usize])
    }

    /// Wraps the x-coordinate around a given width, simulating a toroidal (wrap-around) grid.
    fn wrap_x(&self, width: i32) -> Self {
        Coord {
            x: (self.x + width) % width,
            y: self.y,
        }
    }

    /// Returns a new `Coord` displaced by `dx` and `dy` from the original.
    ///
    /// # Examples
    ///
    /// ```
    /// use hextergen::mesh::Coord;
    ///
    /// let coord = Coord::new(10, 20);
    /// let displaced = coord.displace(3, -2);
    /// assert_eq!(displaced.x, 13);
    /// assert_eq!(displaced.y, 18);
    /// ```
    pub fn displace(&self, dx: i32, dy: i32) -> Self {
        Coord {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

pub struct Hex {
    pub axial: Hexx,
    pub offset: Coord,
    pub center: Vec2,
    pub corners: [Vec2; 6],
    pub neighbors: Vec<(Coord, WrapAround)>,
}

/// Represents a single hexagon in a hexagonal grid system.
///
/// Each `Hex` contains information about its position, both in axial (hexagonal) and offset (rectangular) coordinates,
/// the physical center in 2D space, the positions of its corners, and its neighboring hexagons.
///
/// The axial coordinates (`axial`) use a 2D coordinate system where each axis is skewed by 60 degrees compared to traditional Cartesian coordinates.
/// This is a common system for efficiently handling hexagonal grids.
///
/// # Fields
/// - `axial`: The hex's position in axial coordinates.
/// - `offset`: The hex's position in offset coordinates, which is more intuitive for grid layouts.
/// - `center`: The physical center of the hex in 2D space.
/// - `corners`: The positions of the hex's six corners.
/// - `neighbors`: A vector of neighboring hexagons along with a flag indicating if the neighbor is wrapped around the grid edge.
///
/// # Examples
///
/// Creating a `Hex` and accessing its fields:
///
/// ```
/// use hextergen::mesh::{Coord, Hex};
/// use hexx::{Hex as Hexx,HexLayout, OffsetHexMode};
///
/// let layout = HexLayout::default();
/// let offset_mode = OffsetHexMode::OddColumns;
/// let hex = Hex::new(Coord::new(5, 5), &layout, offset_mode, 10, 10);
///
/// assert_eq!(hex.axial.y, 3, "Axial coordinates do not match");
/// assert_eq!(hex.offset.x, 5, "Offset x-coordinate does not match");
/// assert_eq!(hex.offset.y, 5, "Offset y-coordinate does not match");
/// assert_eq!(hex.center.x, 7.5, "Center x-coordinate does not match");
/// assert_eq!(hex.center.y, -9.526279, "Center y-coordinate does not match");
/// ```
impl Hex {
    /// Creates a new `Hex` based on its offset coordinates, layout, offset mode, and the dimensions of the map.
    ///
    /// The `offset` parameter represents the hex's position in a grid, `layout` and `offset_mode` determine how
    /// hexagons are laid out and converted between coordinate systems, and `map_width` and `map_height`
    /// define the boundaries for neighbor calculation.
    pub fn new(
        offset: Coord,
        layout: &HexLayout,
        offset_mode: OffsetHexMode,
        map_width: i32,
        map_height: i32,
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
                if n_coord.y >= 0 && n_coord.y < map_height {
                    if n_coord.x < 0 || n_coord.x >= map_width {
                        Some((n_coord.wrap_x(map_width), true))
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

/// A trait for creating a grid of `Hex` structs, representing a hexagonal grid system.
///
/// This trait defines a method for generating a 2D array (`ndarray::Array2`) of `Hex`es based on the dimensions
/// of the grid, the layout of the hexagons, and the offset mode. The `new_hexes` function utilizes parallel processing
/// to efficiently create each hexagon in the grid.
trait HexesBuilder {
    /// Generates a new 2D array of `Hex` structs representing the hexagonal grid.
    ///
    /// This method constructs each `Hex` in the grid based on the provided grid dimensions,
    /// hex layout, and offset mode. It leverages parallel processing to improve performance.
    ///
    /// # Parameters
    /// - `map_width`: The width of the grid, in hexes.
    /// - `map_height`: The height of the grid, in hexes.
    /// - `layout`: The layout parameters for the hexes, defining their size and orientation.
    /// - `offset_mode`: The offset mode (even or odd) that affects the calculation of hex positions.
    ///
    /// # Returns
    /// A 2D array (`ndarray::Array2<Hex>`) representing the hexagonal grid.
    fn new_hexes(
        map_width: i32,
        map_height: i32,
        layout: &HexLayout,
        offset_mode: OffsetHexMode,
    ) -> Hexes;
}

impl HexesBuilder for Hexes {
    /// Implementation of `new_hexes` for generating the hexagonal grid.
    fn new_hexes(
        map_width: i32,
        map_height: i32,
        layout: &HexLayout,
        offset_mode: OffsetHexMode,
    ) -> Hexes {
        let builder: Vec<Hex> = (0..map_width)
            .into_par_iter()
            .flat_map(|x| {
                (0..map_height).into_par_iter().map(move |y| {
                    let offset = Coord::new(x, y);
                    Hex::new(offset, layout, offset_mode, map_width, map_height)
                })
            })
            .collect();

        Array2::from_shape_vec((map_width as usize, map_height as usize), builder)
            .expect("Error creating hexes: failed to match grid dimensions with hex count.")
    }
}

pub struct Screen {
    pub displacement: Vec2,
    pub resolution: Vec2,
}

/// Represents the screen space required to display a hexagonal grid.
///
/// This struct calculates and stores the minimum displacement and the resolution necessary to render
/// the entire hexagonal grid within a graphical interface. The `displacement` represents the offset needed
/// to ensure all hexagons are visible on screen, typically by shifting the grid into the positive coordinate space.
/// The `resolution` determines the width and height required to encompass all hexagons.
///
/// # Fields
/// - `displacement`: A `Vec2` representing the minimum x and y offsets required to render the entire grid positively.
/// - `resolution`: A `Vec2` representing the total width and height required to display the grid.
impl Screen {
    /// Constructs a new `Screen` that determines the minimum screen space needed to display a hexagonal grid.
    ///
    /// This method calculates the bounds by processing each hexagon in the grid and its neighbors, finding the
    /// minimum and maximum x and y coordinates. It then determines the displacement and resolution based on these
    /// coordinates to ensure the entire grid can be rendered within a positive coordinate space.
    ///
    /// # Parameters
    /// - `map_width`: The width of the grid, in hexes.
    /// - `map_height`: The height of the grid, in hexes.
    /// - `hexes`: A reference to the `Hexes` representing the hexagonal grid.
    ///
    /// # Returns
    /// A `Screen` instance with calculated displacement and resolution for the grid.
    fn new(map_width: i32, map_height: i32, hexes: &Hexes) -> Self {
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
            &Coord::new(map_width - 1, map_height - 1),
            &mut update_bounds,
        );

        Self {
            displacement: Vec2::new(min_x.abs(), min_y.abs()),
            resolution: Vec2::new(max_x, max_y),
        }
    }
}

pub struct Mesh {
    pub hexes: Hexes,
    pub screen: Screen,
    pub width: i32,
    pub height: i32,
}

/// Represents the entire hexagonal grid system.
///
/// `Mesh` encapsulates the hexagonal grid (`hexes`), the required screen space for rendering (`screen`),
/// and the grid's dimensions. It provides a high-level interface for generating the grid based on specified
/// dimensions and rendering parameters, and for accessing individual hexes within the grid.
///
/// # Fields
/// - `hexes`: The hexagonal grid, represented as a 2D array of `Hex` structs.
/// - `screen`: The calculated screen space required to display the grid, including displacement and resolution.
/// - `width`: The width of the grid, in hexes.
/// - `height`: The height of the grid, in hexes.
///
/// # Examples
///
/// Creating a `Mesh` and retrieving a specific `Hex`:
///
/// ```
/// use hextergen::mesh::Mesh;
/// use hextergen::cmd::GenerateOptions;
///
/// // Assuming `GenerateOptions` is a struct that defines parameters for grid generation,
/// // such as grid dimensions, hex size, orientation, and offset mode.
/// let options = GenerateOptions {
///     width: 10,
///     height: 10,
///     ..Default::default()
///     // Other necessary parameters...
/// };
///
/// let mesh = Mesh::new(&options);
///
/// // Access a specific hex in the grid.
/// let hex = mesh.get_hex(5, 5);
///
/// assert_eq!(hex.axial.y, 5, "Axial coordinates do not match");
/// assert_eq!(hex.offset.x, 5, "Offset x-coordinate does not match");
/// assert_eq!(hex.offset.y, 5, "Offset y-coordinate does not match");
/// assert_eq!(hex.center.x, 77.94228, "Center x-coordinate does not match");
/// assert_eq!(hex.center.y, 75.0, "Center y-coordinate does not match");
/// ```
///
/// This example demonstrates how to create a `Mesh` with specific generation options and how to
/// retrieve a hex at a specific grid location. The `Mesh` struct simplifies the management of the
/// hexagonal grid, allowing easy access to individual hexes and grid properties.
impl Mesh {
    pub fn new(options: &GenerateOptions) -> Self {
        let width = options.width as i32;
        let height = options.height as i32;
        let layout = HexLayout {
            invert_y: true,
            orientation: options.orientation,
            hex_size: Vec2::new(10.0, 10.0),
            ..Default::default()
        };
        let offset_mode = options.offset_mode();
        let hexes = Hexes::new_hexes(width, height, &layout, offset_mode);
        let screen = Screen::new(width, height, &hexes);

        Self {
            hexes,
            screen,
            width,
            height,
        }
    }

    pub fn get_hex(&self, x: i32, y: i32) -> &Hex {
        let coord = Coord::new(x, y);
        &self.hexes[coord.to_dim()]
    }
}
