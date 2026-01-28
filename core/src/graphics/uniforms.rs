#[repr(C, align(16))]
#[derive(Clone, Copy, Debug)]
pub struct CameraData {
    pub position: [f32; 4],
    pub aspect_ratio: f32,
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug)]
pub struct ObjectData {
    pub position: [f32; 4],
    pub scale: f32,
}
