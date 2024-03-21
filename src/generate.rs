use crate::cmd::GenerateOptions;
use crate::mesh::Mesh;
use crate::render;
use crate::topography::Topography;

pub fn generate_map(options: &GenerateOptions) {
    let mesh = Mesh::new(options);
    let topography = Topography::new(options);

    render::quadrants(&mesh, &topography);
}
