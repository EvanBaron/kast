use crate::graphics::instance::Instance;
use crate::graphics::mesh::{Mesh, Vertex};
use crate::graphics::renderer::Renderer;

pub struct Scene {
    pub meshes: Vec<Mesh>,
}

impl Scene {
    pub fn new(renderer: &mut Renderer, instance: &Instance) -> Self {
        let vertices: Vec<Vertex> = vec![
            Vertex {
                position: [-0.5, -0.5, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.0],
            },
        ];

        let indices = vec![0, 1, 2, 1, 3, 2];

        let square = renderer.upload_mesh(instance.physical_device, &vertices, &indices);

        Self {
            meshes: vec![square],
        }
    }
}
