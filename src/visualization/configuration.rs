use types::message::Message;
use regex::Regex;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug)]
struct TopicInfo {
    texture: String,
}

impl TopicInfo {
    fn new() -> TopicInfo {
        TopicInfo {
            texture: String::new(),
        }
    }

    fn set_texture(&mut self, filename: &str) {
        self.texture = String::from(filename);

    }
}

pub struct Configuration {
    config_file: String,
    topics: HashMap<String, TopicInfo>,
    textures: Vec<String>,
}

impl Configuration {
    fn load_config_file(&mut self, filename: String) {
        self.textures = vec![];

        let topic_re = Regex::new(r"#\s+([^\s]+)[^#]*").unwrap();
        let rule_re = Regex::new(r"([^:\s]+)\s*:\s*([^:\s]+)").unwrap();

        let mut topics = HashMap::<String, TopicInfo>::new();

        if let Ok(mut file) = File::open(filename) {
            let mut contents = String::new();
            if let Ok(_) = file.read_to_string(&mut contents) {
                for topic in topic_re.captures_iter(&contents) {
                    let topic_name = topic.get(1).unwrap().as_str();
                    let mut topic_data = TopicInfo::new();

                    for rule in rule_re.captures_iter(topic.get(0).unwrap().as_str()) {
                        let name  = rule.get(1).unwrap().as_str();
                        let value = rule.get(2).unwrap().as_str();

                        match name {
                            "texture" => {
                                topic_data.set_texture(value);
                                self.textures.push(String::from(value));
                            },
                            _         => {},
                        };
                    }

                    topics.insert(String::from(topic_name), topic_data);
                }
            }
        };

        self.topics = topics;
    }

    pub fn new(filename: String) -> Configuration {
        let mut new_configuration = Configuration {
            config_file: filename.clone(),
            topics: HashMap::<String, TopicInfo>::new(),
            textures: vec![],
        };
        new_configuration.load_config_file(filename);

        new_configuration
    }

    pub fn get_texture_names(&self) -> Vec<String> {
        return self.textures.clone();
    }

    pub fn get_render_info(&self, msg: &Message) -> (String, Vec<(f32, f32, f32)>) {
        let texture_name = match self.topics.get(&msg.topic) {
            Some(info) => info.texture.clone(),
            None       => String::from(""),
        };

        let mut x_idx: Option<usize> = None;
        let mut y_idx: Option<usize> = None;
        let mut z_idx: Option<usize> = None;

        for (i, field) in msg.format.iter().enumerate() {
            match field.as_ref() {
                "x" => { x_idx = Some(i) },
                "y" => { y_idx = Some(i) },
                "z" => { z_idx = Some(i) },
                _   => {},
            };
        }

        let mut positions = Vec::<(f32, f32, f32)>::new();
        for object in &msg.objects {
            let (mut x, mut y, mut z) = (0.0, 0.0, 0.0);
            if let Some(idx) = x_idx { x = object[idx].parse().unwrap(); }
            if let Some(idx) = y_idx { y = object[idx].parse().unwrap(); }
            if let Some(idx) = z_idx { z = object[idx].parse().unwrap(); }

            positions.push((x, y, z));
        }

        return (texture_name, positions);
    }
}
