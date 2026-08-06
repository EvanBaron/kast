use kast::prelude::*;

const TRIANGLE_VERT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/triangle.vert.spv"));
const TRIANGLE_FRAG: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/triangle.frag.spv"));

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

const TRIANGLE_VERTICES: [Vertex; 3] = [
    Vertex {
        position: [0.0, -0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5],
        color: [0.0, 0.0, 1.0],
    },
];

fn main() {
    App::builder()
        .with_window(WindowConfig {
            title: "Default Window".to_owned(),
            size: PhysicalSize {
                width: 800,
                height: 600,
            },
            position: None,
            mode: WindowMode::Windowed,
        })
        .build_with(DefaultExperience::default())
        .run();
}

#[derive(Default)]
struct DefaultExperience {
    pipeline: Option<PipelineHandle>,
    vertex_buffer: Option<BufferHandle>,
}

impl AppState for DefaultExperience {
    fn on_resume(&mut self, context: &mut AppContext) {
        if self.pipeline.is_some() {
            return;
        }

        let Some(gfx) = context.renderer.context_mut() else {
            return;
        };

        let vertex_layout = [
            VertexAttribute {
                format: VertexFormat {
                    size: 8,
                    components: 2,
                },
                offset: 0,
            },
            VertexAttribute {
                format: VertexFormat {
                    size: 12,
                    components: 3,
                },
                offset: 8,
            },
        ];

        let pipeline = gfx
            .create_pipeline(&PipelineDescriptor {
                vertex_shader: TRIANGLE_VERT,
                fragment_shader: TRIANGLE_FRAG,
                vertex_layout: &vertex_layout,
                cull_mode: CullMode::None,
                ..Default::default()
            })
            .expect("failed to create triangle pipeline");

        let buffer_size = core::mem::size_of_val(&TRIANGLE_VERTICES) as u64;
        let vertex_buffer = gfx
            .create_buffer(&BufferDescriptor {
                size: buffer_size,
                usage: BufferUsage::Vertex,
                memory_properties: MemoryProperties::HostVisible,
            })
            .expect("failed to create vertex buffer");

        let vertex_bytes = unsafe {
            core::slice::from_raw_parts(TRIANGLE_VERTICES.as_ptr() as *const u8, buffer_size as usize)
        };
        gfx.upload_buffer(vertex_buffer, vertex_bytes)
            .expect("failed to upload vertex data");

        self.pipeline = Some(pipeline);
        self.vertex_buffer = Some(vertex_buffer);
    }

    fn on_render(&mut self, context: &mut AppContext) {
        let (Some(pipeline), Some(vertex_buffer)) = (self.pipeline, self.vertex_buffer) else {
            return;
        };
        let Some(gfx) = context.renderer.context_mut() else {
            return;
        };

        let mut pass = RenderPass::new(RenderPassDescriptor {
            color_attachments: vec![ColorAttachment {
                target: None,
                clear_color: Some([0.02, 0.02, 0.05, 1.0]),
            }],
            depth_attachment: None,
        });

        pass.draw_list.push(DrawCall {
            pipeline,
            vertex_buffer,
            index_buffer: None,
            index_count: 3,
            instance_count: 1,
            push_constants: None,
        });

        if let Err(error) = gfx.submit(&[pass]) {
            eprintln!("Failed to submit frame: {error}");
        }
    }
}
