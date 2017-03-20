#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct Message {
    pub publisher: String,
    pub topic: String,
    pub timestamp: u64,
    pub format: Vec<String>,
    pub objects: Vec<Vec<String>>,
}
