use types::message::MessageIn;
use types::{Geometry, ObjectRenderInfo};
use regex::Regex;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug)]
struct TypeInfo {
    texture: String,
    model:   Geometry,
}

impl TypeInfo {
    fn new() -> TypeInfo {
        TypeInfo {
            texture: String::from(""),
            model:   Geometry::Square,
        }
    }

    fn set_texture(&mut self, filename: &str) {
        self.texture = String::from(filename);
    }

    fn set_model(&mut self, model: &str) {
        match model {
            "square" => self.model = Geometry::Square,
            "cube"   => self.model = Geometry::Cube,
            &_       => {},
        };
    }
}

pub struct Configuration {
    config_file: String,
    types:       HashMap<String, TypeInfo>,
    textures:    Vec<String>,
}

impl Configuration {
    fn load_config_file(&mut self, filename: String) {
        self.textures = vec![];

        let type_re = Regex::new(r"#\s+([^\s]+)[^#]*").unwrap();
        let rule_re = Regex::new(r"([^:\s]+)\s*:\s*([^:\s]+)").unwrap();

        let mut types = HashMap::<String, TypeInfo>::new();

        if let Ok(mut file) = File::open(filename) {
            let mut contents = String::new();
            if let Ok(_) = file.read_to_string(&mut contents) {
                for type_description in type_re.captures_iter(&contents) {
                    let type_name = type_description.get(1).unwrap().as_str();
                    let mut type_data = TypeInfo::new();

                    for rule in rule_re.captures_iter(type_description.get(0).unwrap().as_str()) {
                        let name  = rule.get(1).unwrap().as_str();
                        let value = rule.get(2).unwrap().as_str();

                        match name {
                            "texture"  => {
                                type_data.set_texture(value);
                                self.textures.push(String::from(value));
                            },
                            "geometry" => {
                                type_data.set_model(value);
                            }
                            _          => {},
                        };
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
        };
        new_configuration.load_config_file(filename);

        new_configuration
    }

    pub fn get_texture_names(&self) -> Vec<String> {
        return self.textures.clone();
    }

    pub fn get_render_info(&self, msg: &MessageIn) -> Vec<ObjectRenderInfo> {
        let empty_str = String::new();
        let default_info = TypeInfo::new();

        msg.objects.iter().map(|obj: &HashMap<String, String>| {
            let id = obj.get("id").unwrap_or(&empty_str).parse::<u32>().unwrap_or(u32::max_value());
            let x = obj.get("x").unwrap_or(&empty_str).parse::<f32>().unwrap_or(0.0);
            let y = obj.get("y").unwrap_or(&empty_str).parse::<f32>().unwrap_or(0.0);
            let z = obj.get("z").unwrap_or(&empty_str).parse::<f32>().unwrap_or(0.0);

            let type_info = match obj.get("type") {
                Some(type_name) => self.types.get(type_name),
                None            => None,
            }.unwrap_or(&default_info);

            ObjectRenderInfo {
                id:           id,
                model:        type_info.model.clone(),
                texture_name: type_info.texture.clone(),
                position:     (x, y, z),
            }
        }).collect()
    }
}
