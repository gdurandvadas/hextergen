use crate::cmd::GenerateOptions;
use crate::mesh::Mesh;
use crate::render;

pub fn generate_map(options: &GenerateOptions) {
    let mesh = Mesh::new(options);
    render::quadrants(&mesh);
}
