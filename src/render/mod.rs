mod colors;
use crate::mesh::{Coord, Hex, Mesh, Screen};
use crate::topography::Topography;
use hexx::Vec2;
use image::{ImageBuffer, Rgba};
use imageproc::drawing::{draw_filled_circle_mut, draw_line_segment_mut, draw_polygon_mut};
use imageproc::point::Point;
use rayon::prelude::*;

use self::colors::Colors;

#[derive(Debug)]
struct Polygon {
    center: Point<f32>,
    corners: [Point<f32>; 6],
    color: Rgba<u8>,
}

impl Polygon {
    fn new(hex: &Hex, color: Rgba<u8>, displacement: &Point<f32>) -> Self {
        let convert_point =
            |point: &Vec2| Point::new(point.x + displacement.x, point.y + displacement.y);

        let center = convert_point(&hex.center);
        let corners = [
            convert_point(&hex.corners[0]),
            convert_point(&hex.corners[1]),
            convert_point(&hex.corners[2]),
            convert_point(&hex.corners[3]),
            convert_point(&hex.corners[4]),
            convert_point(&hex.corners[5]),
        ];

        Self {
            center,
            corners,
            color,
        }
    }

    fn corners(&self) -> Vec<Point<i32>> {
        self.corners
            .iter()
            .map(|point| Point::new(point.x as i32, point.y as i32))
            .collect()
    }

    fn center(&self) -> Point<i32> {
        Point::new(self.center.x as i32, self.center.y as i32)
    }
}

type Polygons = Vec<Polygon>;

enum Quadrant {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Quadrant {
    fn to_string(&self) -> String {
        match self {
            Self::TopLeft => "top_left".to_owned(),
            Self::TopRight => "top_right".to_owned(),
            Self::BottomLeft => "bottom_left".to_owned(),
            Self::BottomRight => "bottom_right".to_owned(),
        }
    }

    fn mesh(&self, center: &Coord, width: i32, height: i32) -> (Coord, Coord) {
        match self {
            Quadrant::TopLeft => (Coord::new(0, 0), center.displace(0, 1)),
            Quadrant::TopRight => (Coord::new(center.x - 1, 0), Coord::new(width, center.y + 1)),
            Quadrant::BottomLeft => (Coord::new(0, center.y - 1), Coord::new(center.x, height)),
            Quadrant::BottomRight => (
                Coord::new(center.x - 1, center.y - 1),
                Coord::new(width, height),
            ),
        }
    }

    fn displacement(&self, center: &Vec2, screen: &Screen) -> Vec2 {
        match self {
            Quadrant::TopLeft => screen.displacement + Vec2::new(0.0, 0.0),
            Quadrant::TopRight => screen.displacement + Vec2::new(center.x * -1.0, 0.0),
            Quadrant::BottomLeft => Vec2::new(screen.displacement.x, center.y * -1.0),
            Quadrant::BottomRight => {
                Vec2::new(screen.displacement.x + center.x * -1.0, center.y * -1.0)
            }
        }
    }

    fn resolution(&self, center: &Vec2, screen: &Screen) -> Point<f32> {
        match self {
            Quadrant::TopLeft => {
                let width = center.x;
                let height = center.y + screen.displacement.y;
                Point::new(width, height)
            }
            Quadrant::TopRight => {
                let width = screen.resolution.x - center.x + screen.displacement.x;
                let height = center.y + screen.displacement.y;
                Point::new(width, height)
            }
            Quadrant::BottomLeft => {
                let width = center.x;
                let height = screen.resolution.y - center.y;
                Point::new(width, height)
            }
            Quadrant::BottomRight => {
                let width = screen.resolution.x - center.x + screen.displacement.x;
                let height = screen.resolution.y - center.y;
                Point::new(width, height)
            }
        }
    }

    fn render(&self, mesh: &Mesh, topography: &Topography, center: &Hex) {
        let (start, end) = self.mesh(&center.offset, mesh.width as i32, mesh.height as i32);
        let relative_displacement = self.displacement(&center.center, &mesh.screen);
        let resolution = self.resolution(&center.center, &mesh.screen);
        let displacement = Point::new(relative_displacement.x, relative_displacement.y);

        let polygons: Polygons = (start.x..end.x)
            .into_par_iter()
            .flat_map(|x| {
                (start.y..end.y).into_par_iter().map(move |y| {
                    let hex = mesh.get_hex(x, y);
                    let mut color = colors::Debug::Red.rgba();
                    if topography.seeds.contains(&hex.offset) {
                        color = colors::Debug::Green.rgba();
                    } else {
                        let elevation = topography.get_hex(x, y);
                        color = colors::Debug::from_elevation(elevation);
                    }
                    Polygon::new(hex, color, &displacement)
                })
            })
            .collect();

        let mut img =
            ImageBuffer::from_pixel(resolution.x as u32, resolution.y as u32, Rgba([0, 0, 0, 0]));

        polygons.iter().for_each(|polygon| {
            draw_polygon_mut(&mut img, &polygon.corners(), polygon.color);
            // draw_filled_circle_mut(
            //     &mut img,
            //     (polygon.center.x, polygon.center.y),
            //     2,
            //     Rgba([0, 0, 0, 120]),
            // );

            // for i in 0..6 {
            //     let corner_a = polygon.corners[i];
            //     let corner_a = (corner_a.x, corner_a.y);

            //     let next = (i + 1) % 6;
            //     let corner_b = polygon.corners[next];
            //     let corner_b = (corner_b.x, corner_b.y);

            //     draw_line_segment_mut(&mut img, corner_a, corner_b, Rgba([0, 0, 0, 120]));
            // }
        });

        let file_name = format!("_debug_{}.png", self.to_string());
        img.save(file_name).unwrap();
    }
}

pub fn quadrants(mesh: &Mesh, topography: &Topography) {
    let center = mesh.get_hex((mesh.width / 2) as i32, (mesh.height / 2) as i32);
    let quadrants = [
        Quadrant::TopLeft,
        Quadrant::TopRight,
        Quadrant::BottomLeft,
        Quadrant::BottomRight,
    ];
    quadrants.par_iter().for_each(|quadrant| {
        quadrant.render(mesh, topography, center);
    });
}
