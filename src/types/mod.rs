pub mod message;

pub enum Geometry {
    Square,
    Cube,
}

pub struct ObjectRenderInfo {
    pub id:           u32,
    pub model:        Geometry,
    pub texture_name: String,
    pub position:     (f32, f32, f32),
}
