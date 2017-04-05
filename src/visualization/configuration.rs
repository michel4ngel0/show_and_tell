use types::message::{MessageIn, Object};
use types::{Geometry, ObjectRenderInfo};
use regex::Regex;

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::prelude::*;

use glutin::VirtualKeyCode;

#[derive(Debug)]
struct TypeInfo {
    texture: String,
    model:   Geometry,
    keys:    HashSet<String>,
    color:   (u8, u8, u8),
}

impl TypeInfo {
    fn new() -> TypeInfo {
        TypeInfo {
            texture: String::from(""),
            model:   Geometry::Square,
            keys:    HashSet::<String>::new(),
            color:   (30, 30, 30),
        }
    }

    fn set_texture(&mut self, filename: &str) {
        self.texture = String::from(filename);
    }

    fn set_model(&mut self, model: &str) {
        match model {
            "square" => self.model = Geometry::Square,
            "cube"   => self.model = Geometry::Cube,
            _       => {},
        };
    }

    fn add_keys(&mut self, keys: Vec<&str>) {
        let keys: Vec<String> = keys.into_iter().map(move |s| String::from(s)).collect();
        self.keys.extend(keys);
    }

    fn set_color(&mut self, red: &str, green: &str, blue: &str) {
        let red = red.parse::<u8>();
        let green = green.parse::<u8>();
        let blue = blue.parse::<u8>();

        if red.is_err() || green.is_err() || blue.is_err() {
            return;
        }

        self.color = (red.unwrap(), green.unwrap(), blue.unwrap());
    }
}

pub struct Configuration {
    config_file: String,
    types:       HashMap<String, TypeInfo>,
    textures:    Vec<String>,
    key_map:     HashMap<VirtualKeyCode, String>,
}

impl Configuration {
    fn load_key_map() -> HashMap<VirtualKeyCode, String> {
        use glutin::VirtualKeyCode::*;

        let key_map_vec: Vec<(VirtualKeyCode, &'static str)> = include!("key_map.txt");

        let mut key_map = HashMap::<VirtualKeyCode, String>::new();
        for (code, attribute) in key_map_vec {
            key_map.insert(code, String::from(attribute));
        }

        key_map
    }

    fn load_config_file(&mut self, filename: String) {
        self.textures = vec![];

        let section_re  = Regex::new(r"#(?:[^#]*)").unwrap();
        let header_re   = Regex::new(r"#(TYPE)(?:\s*)(\S*)").unwrap();
        let rule_re     = Regex::new(r"(\S+)(?:\s*):((?:\s*\S+)+)").unwrap();
        let argument_re = Regex::new(r"\S+").unwrap();

        let mut types = HashMap::<String, TypeInfo>::new();

        if let Ok(mut file) = File::open(filename) {
            let mut contents = String::new();
            if let Ok(_) = file.read_to_string(&mut contents) {

                for section_match in section_re.find_iter(&contents) {
                    let section = section_match.as_str();

                    let mut type_name = "";
                    let mut type_data = TypeInfo::new();

                    let lines = section.split('\n').map(|s| s.trim()).filter(|s| s.len() > 0);
                    for line in lines {
                        if let Some(header) = header_re.captures(&line) {
                            type_name = header.get(2).unwrap().as_str();
                        } else if let Some(rule) = rule_re.captures(&line) {
                            let attribute = rule.get(1).unwrap().as_str();
                            let arguments_str = rule.get(2).unwrap().as_str();

                            let args: Vec<&str> = argument_re.find_iter(&arguments_str)
                                .map(|s| s.as_str())
                                .collect();

                            match (attribute, args.len()) {
                                ("model", 1)   => type_data.set_model(args[0]),
                                ("color", 3)   => type_data.set_color(args[0], args[1], args[2]),
                                ("texture", 1) => {
                                    type_data.set_texture(args[0]);
                                    self.textures.push(String::from(args[0]));
                                },
                                ("key", _)     => type_data.add_keys(args),
                                _              => println!("Invalid rule: {}", line),
                            }
                        }
                    }

                    types.insert(String::from(type_name), type_data);
                }
            }

        };

        self.textures.sort();
        self.textures.dedup();

        self.types = types;
    }

    pub fn new(filename: String) -> Configuration {
        let mut new_configuration = Configuration {
            config_file: filename.clone(),
            types:       HashMap::<String, TypeInfo>::new(),
            textures:    vec![],
            key_map:     Configuration::load_key_map(),
        };
        new_configuration.load_config_file(filename);

        new_configuration
    }

    pub fn get_texture_names(&self) -> Vec<String> {
        return self.textures.clone();
    }

    pub fn parse_message(&self, msg: &MessageIn) -> (Vec<ObjectRenderInfo>, HashMap<u32, Object>, String) {
        let empty_str = String::new();
        let default_info = TypeInfo::new();

        let message_id = msg.id.clone();

        let (render_info, details): (Vec<ObjectRenderInfo>, Vec<Option<(u32, Object)>>) = msg.objects.iter()
            .map(|obj: &Object| -> (ObjectRenderInfo, Option<(u32, Object)>) {
                let id = obj.get("id").unwrap_or(&empty_str).parse::<u32>().unwrap_or(u32::max_value());
                let x = obj.get("x").unwrap_or(&empty_str).parse::<f32>().unwrap_or(0.0);
                let y = obj.get("y").unwrap_or(&empty_str).parse::<f32>().unwrap_or(0.0);
                let z = obj.get("z").unwrap_or(&empty_str).parse::<f32>().unwrap_or(0.0);

                let type_info = match obj.get("type") {
                    Some(type_name) => self.types.get(type_name),
                    None            => None,
                }.unwrap_or(&default_info);

                let (r, g, b) = type_info.color;

                let info = ObjectRenderInfo {
                    id:           id,
                    model:        type_info.model.clone(),
                    texture_name: type_info.texture.clone(),
                    color:        (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0),
                    position:     (x, y, z),
                };

                let details_option = if id == u32::max_value() { None } else { Some((id, obj.clone())) };

                (info, details_option)
            })
            .unzip();

        let objects_vec: Vec<(u32, Object)> = details.into_iter()
            .filter(|option| option.is_some())
            .map(|option| option.unwrap())
            .collect();

        let mut objects = HashMap::<u32, Object>::new();
        for (id, object) in objects_vec {
            objects.insert(id, object);
        }

        (render_info, objects, message_id)
    }

    pub fn get_key_str(&self, code: VirtualKeyCode, attributes: &Object) -> Option<String> {
        let key_name = self.key_map.get(&code)
            .expect("Unknown key pressed");
        let type_info = match attributes.get("type") {
            None       => None,
            Some(info) => self.types.get(info),
        };

        match type_info {
            None       => None,
            Some(info) => if info.keys.contains(key_name) { Some(key_name.clone()) } else { None },
        }
    }
}
