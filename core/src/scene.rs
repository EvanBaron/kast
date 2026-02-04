use crate::graphics::image::{BYTES_PER_PIXEL, ImageData};
use crate::graphics::instance::Instance;
use crate::graphics::mesh::{Mesh, Vertex};
use crate::graphics::renderer::Renderer;
use crate::graphics::uniforms::{CameraData, ObjectData};

const SQUARE_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, -1.0, 0.0],
        texture_coords: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
        texture_coords: [1.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, 0.0],
        texture_coords: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        texture_coords: [1.0, 1.0],
    },
];

const TRIANGLE_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, -1.0, 0.0],
        texture_coords: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
        texture_coords: [1.0, 0.0],
    },
    Vertex {
        position: [0.0, 1.0, 0.0],
        texture_coords: [0.5, 1.0],
    },
];

const SQUARE_INDICES: &[u32] = &[0, 1, 2, 1, 3, 2];
const TRIANGLE_INDICES: &[u32] = &[0, 1, 2];

pub struct Entity {
    pub mesh: Mesh,
    pub data: ObjectData,
}

pub struct Scene {
    pub camera_data: CameraData,
    pub entities: Vec<Entity>,
}

impl Scene {
    pub fn get_textures() -> Vec<ImageData> {
        let mut image_data_array = [
            ImageData {
                bytes_per_pixel: BYTES_PER_PIXEL,
                width: 4,
                height: 4,
                data: Vec::new(),
            },
            ImageData {
                bytes_per_pixel: BYTES_PER_PIXEL,
                width: 8,
                height: 8,
                data: Vec::new(),
            },
        ];

        // Fill image data
        image_data_array[0].data = vec![0xFF; image_data_array[0].get_data_size()];
        for i in 0..image_data_array[0].height {
            for j in 0..image_data_array[0].width {
                let pixel_data = image_data_array[0].get_pixel(i, j);
                pixel_data[0] = if i % 2 == 0 { 0xA0 } else { 0xFF };
                pixel_data[1] = if j % 2 == 0 { 0xA0 } else { 0xFF };
                pixel_data[2] = 0xA0;
                pixel_data[3] = 0xFF;
            }
        }

        image_data_array[1].data = vec![0xFF; image_data_array[1].get_data_size()];
        for i in 0..image_data_array[1].height {
            for j in 0..image_data_array[1].width {
                let pixel_data = image_data_array[1].get_pixel(i, j);
                pixel_data[0] = if i % 2 == 0 { 0xC0 } else { 0xFF };
                pixel_data[1] = if j % 2 == 0 { 0xC0 } else { 0xFF };
                pixel_data[2] = 0xFF;
                pixel_data[3] = 0xFF;
            }
        }

        image_data_array.into()
    }

    pub fn new(renderer: &mut Renderer, instance: &Instance) -> Self {
        let camera_data = CameraData {
            position: [0.0, 0.0, -0.5, 0.0],
            aspect_ratio: 1.0, // Redefined after
        };

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
                texture_id: 1,
            },
        };

        let entity2 = Entity {
            mesh: triangle.clone(),
            data: ObjectData {
                position: [0.25, -0.25, 0.0, 0.0],
                scale: 0.25,
                texture_id: 0,
            },
        };

        let entity3 = Entity {
            mesh: triangle.clone(),
            data: ObjectData {
                position: [0.25, 0.25, 0.0, 0.0],
                scale: 0.25,
                texture_id: 0,
            },
        };

        Self {
            camera_data,
            entities: vec![entity1, entity2, entity3],
        }
    }
}
