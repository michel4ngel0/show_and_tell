use std::collections::HashMap;

#[derive(RustcDecodable, RustcEncodable)]
pub struct Message {
    pub publisher: String,
    pub topic: String,
    pub timestamp: u64,
    pub objects: Vec<HashMap<String, String>>,
}
