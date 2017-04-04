use std::collections::HashMap;

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct MessageIn {
    pub publisher: String,
    pub id: String,
    pub objects: Vec<HashMap<String, String>>,
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct MessageOut {
    pub publisher: String,
}
