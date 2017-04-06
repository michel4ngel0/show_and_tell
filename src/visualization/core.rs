use types::message::{MessageIn, MessageOut, Object};
use types::ObjectRenderInfo;
use types::double_channel::Endpoint;
use visualization::camera::Camera;
use visualization::configuration::Configuration;
use visualization::render::Renderer;

use glutin;
use glutin::ElementState;
use gl;
use cgmath;

use std::collections::HashMap;
use std::time::{Instant};
use std::f64::consts::PI;
use std::cmp::max;

pub struct Visualization {
    link_core:     Endpoint<Option<MessageOut>, Option<MessageIn>>,
    publisher:     String,
    configuration: Configuration,
}

impl Visualization {
    pub fn new(link: Endpoint<Option<MessageOut>, Option<MessageIn>>, publisher: String, config_file: String) -> Visualization {
        Visualization {
            link_core:     link,
            publisher:     publisher,
            configuration: Configuration::new(config_file),
        }
    }

    pub fn run(&mut self) {
        let window_name = format!("[{}]", self.publisher);
        let (mut window_x, mut window_y) = (800, 600);

        let window = glutin::WindowBuilder::new()
            .with_title(window_name)
            .with_dimensions(window_x, window_y)
            .with_vsync()
            .with_gl(glutin::GlRequest::Latest)
            .build_strict()
            .expect("Failed to open the window");
        unsafe { window.make_current() }
            .expect("Failed to set current context");

        //  Initialize OpenGL
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

        let textures_names = self.configuration.get_texture_names();
        let mut renderer = Renderer::new(window_x as usize, window_y as usize, textures_names);

        //  Camera position
        let mut camera = Camera::new(cgmath::Point3::<f32>::new(0.0, 0.0, 10.0));

        let time_start = Instant::now();

        let mut active_object: Option<u32> = None;

        let mut last_message_id: Option<String> = None;
        let mut objects: HashMap<u32, Object> = HashMap::<u32, Object>::new();
        let mut render_info: Vec<ObjectRenderInfo> = vec![];
        let mut permanent_info: HashMap<u32, ObjectRenderInfo> = HashMap::<u32, ObjectRenderInfo>::new();

        let mut mouse_x = 0;
        let mut mouse_y = 0;
        let mut mouse_pos = cgmath::Point2::<f32>::new(0.0, 0.0);

        let mut is_left_pressed   = false;
        let mut is_right_pressed  = false;
        let mut is_middle_pressed = false;

        'main: loop {
            if let Ok(msg_option) = self.link_core.try_recv() {
                match msg_option {
                    Some(msg) => {
                        let parsed_message = self.configuration.parse_message(&msg);
                        render_info     = parsed_message.0;
                        objects         = parsed_message.1;
                        last_message_id = Some(parsed_message.2);
                    },
                    None      => {
                        println!("(visualization) terminating");
                        break 'main;
                    },
                };
            }

            for object in render_info.clone() {
                if let Some(id) = object.permanent_id { let _ = permanent_info.insert(id, object.clone()); }
            }
            render_info = render_info.into_iter().filter(|object| object.permanent_id.is_none() ).collect();

            let time_now = Instant::now();
            let time_from_start = time_now - time_start;

            for event in window.poll_events() {
                use glutin::Event::*;

                match event {
                    Closed => {
                        break 'main
                    },
                    Resized(x, y) => {
                        window_x = x;
                        window_y = y;
                        renderer.resize(x as usize, y as usize);
                    },

                    MouseWheel(glutin::MouseScrollDelta::LineDelta(_, y), _) => {
                        camera.zoom(y);
                    },
                    MouseMoved(x, y) => {
                        let prev_pos = mouse_pos;

                        mouse_x = x as i32;
                        mouse_y = window_y as i32 - y - 1;

                        mouse_pos = cgmath::Point2::<f32>::new(mouse_x as f32, mouse_y as f32);

                        if is_right_pressed {
                            camera.step(mouse_pos - prev_pos);
                        }
                        // if is_middle_pressed {
                        //     camera.turn_around(mouse_pos - prev_pos);
                        // }
                    },
                    MouseInput(state @ _, button @ _) => {
                        use glutin::MouseButton::*;
                        use glutin::ElementState::*;

                        let value = state == Pressed;
                        match button {
                            Left   => is_left_pressed   = value,
                            Right  => is_right_pressed  = value,
                            Middle => is_middle_pressed = value,
                            _      => {},
                        };

                        if state == Pressed && button == Left {
                            let id = renderer.get_id((mouse_x as usize, mouse_y as usize));
                            active_object = id;
                        }
                    },

                    KeyboardInput(ElementState::Pressed, _, Some(code)) => {
                        if let Some(object_id) = active_object {
                            match objects.get(&object_id) {
                                None             => {},
                                Some(attributes) => {
                                    let maybe_code_str = self.configuration.get_key_str(code, attributes);

                                    if let Some(code_str) = maybe_code_str {
                                        if let Some(id) = last_message_id.clone() {
                                            let _ = self.link_core.send(Some(
                                                MessageOut {
                                                    publisher: self.publisher.clone(),
                                                    id:        id,
                                                    object_id: object_id,
                                                    key_code:  code_str,
                                                }
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    },

                    _ => {},
                }
            }

            //  Calculate uniform tranformations
            let (window_x, window_y) = window.get_inner_size().unwrap();
            let aspect_ratio = (window_x as f32) / (window_y as f32);

            let camera_transformation = camera.get_matrix();
            let proj = cgmath::perspective(cgmath::Deg(50.0f32), aspect_ratio, 0.01, 1000.0);
            let camera_projection = (proj * camera_transformation).into();

            let phi = (time_from_start.as_secs() as f64 + ((time_from_start.subsec_nanos() as f64) / 1000000000.0)) % (2.0 * PI);
            let mut strings: Vec<String> = vec![];
            if let Some(id) = active_object {
                if let Some(object) = objects.get(&id) {
                    strings = sort_stats(&object);
                }
            }
            renderer.render(&render_info, &permanent_info, camera_projection, active_object, strings, phi);

            window.swap_buffers()
                .expect("Failed to swap buffers");
        }

        let _ = self.link_core.send(None);
    }
}

fn sort_stats(object: &Object) -> Vec<String> {
    use std::cmp::Ordering::*;
    use std::ops::Deref;

    let mut stats: Vec<(&String, &String)> = object.iter().collect();
    stats.sort_by(|lhs, rhs| {
        match (lhs.0.as_str(), rhs.0.as_str()) {
            ("id", _)   => Less,
            (_, "id")   => Greater,
            ("type", _) => Less,
            (_, "type") => Greater,
            (_, _)      => lhs.partial_cmp(&rhs).unwrap(),
        }
    });

    let mut longest = 0;
    for &(attribute, _) in &stats { longest = max(longest, attribute.len()); }

    stats.iter()
        .map(|&(attribute, value)| format!("{:len$}: {}", attribute.clone(), value, len = longest) )
        .collect()
}
