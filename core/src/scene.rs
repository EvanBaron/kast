use crate::graphics::instance::Instance;
use crate::graphics::mesh::{Mesh, Vertex};
use crate::graphics::renderer::Renderer;
use crate::graphics::uniforms::ObjectData;

const SQUARE_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, -1.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
    },
];

const TRIANGLE_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, -1.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
    },
    Vertex {
        position: [0.0, 1.0, 0.0],
    },
];

const SQUARE_INDICES: &[u32] = &[0, 1, 2, 1, 3, 2];
const TRIANGLE_INDICES: &[u32] = &[0, 1, 2];

pub struct Entity {
    pub mesh: Mesh,
    pub data: ObjectData,
}

pub struct Scene {
    pub entities: Vec<Entity>,
}

impl Scene {
    pub fn new(renderer: &mut Renderer, instance: &Instance) -> Self {
        let square =
            renderer.upload_mesh(instance.physical_device, &SQUARE_VERTICES, SQUARE_INDICES);
        let triangle = renderer.upload_mesh(
            instance.physical_device,
            &TRIANGLE_VERTICES,
            TRIANGLE_INDICES,
        );

        let entity1 = Entity {
            mesh: square.clone(),
            data: ObjectData {
                position: [-0.25, 0.25, 0.0, 0.0],
                scale: 0.25,
            },
        };

        let entity2 = Entity {
            mesh: triangle.clone(),
            data: ObjectData {
                position: [0.25, -0.25, 0.0, 0.0],
                scale: 0.25,
            },
        };

        let entity3 = Entity {
            mesh: triangle.clone(),
            data: ObjectData {
                position: [0.25, 0.25, 0.0, 0.0],
                scale: 0.25,
            },
        };

        Self {
            entities: vec![entity1, entity2, entity3],
        }
    }
}
