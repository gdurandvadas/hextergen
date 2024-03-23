use log::{debug, info};

use crate::cmd::GenerateOptions;
use crate::mesh::Mesh;
use crate::render;
use crate::topography::Topography;
use crate::utils::id;

pub fn generate_map(options: &GenerateOptions) {
    let id = id::new(options.seed, 8);
    info!("Generating new map ID: {}", id);


    let start = std::time::Instant::now();
    let mesh = Mesh::new(options);
    info!("Mesh generated in {}ms", start.elapsed().as_millis());

    let start = std::time::Instant::now();
    let topography = Topography::new(options, &mesh);
    info!("Topography generated in {}ms", start.elapsed().as_millis());

    let start = std::time::Instant::now();
    render::quadrants(&mesh, &topography);
    info!("Rendered quadrants in {}ms", start.elapsed().as_millis());
}
