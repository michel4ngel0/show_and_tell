pub mod message;
pub mod double_channel;

#[derive(Debug, Clone)]
pub enum Geometry {
    Square,
    Cube,
}

#[derive(Debug, Clone)]
pub struct ObjectRenderInfo {
    pub id:           u32,
    pub permanent_id: Option<u32>,
    pub model:        Geometry,
    pub texture_name: String,
    pub color:        (f32, f32, f32),
    pub position:     (f32, f32, f32),
}
