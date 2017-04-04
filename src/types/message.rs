use std::collections::HashMap;

pub type Object = HashMap<String, String>;

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct MessageIn {
    pub publisher: String,
    pub id: String,
    pub objects: Vec<Object>,
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct MessageOut {
    pub publisher: String,
    pub id: String,
    pub object_id: u32,
    pub key_code: String,
}
