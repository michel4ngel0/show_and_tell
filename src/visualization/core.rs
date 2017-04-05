use types::message::{MessageIn, MessageOut, Object};
use types::ObjectRenderInfo;
use types::double_channel::Endpoint;
use visualization::camera::Camera;
use visualization::configuration::Configuration;
use visualization::render::Renderer;

use glutin;
use glutin::ElementState;
use gl;
use image;
use cgmath;

use std::collections::HashMap;

pub struct Visualization {
    link_core:     Endpoint<Option<MessageOut>, Option<MessageIn>>,
    publisher:     String,
    configuration: Configuration,
    renderer:      Renderer,
}

impl Visualization {
    pub fn new(link: Endpoint<Option<MessageOut>, Option<MessageIn>>, publisher: String, config_file: String) -> Visualization {
        Visualization {
            link_core:     link,
            publisher:     publisher,
            configuration: Configuration::new(config_file),
            renderer:      Renderer::new(),
        }
    }

    pub fn run(&mut self) {
        let window_name = format!("[{}]", self.publisher);

        let window = glutin::WindowBuilder::new()
            .with_title(window_name)
            .with_dimensions(800, 600)
            .with_vsync()
            .with_gl(glutin::GlRequest::Latest)
            .build_strict()
            .expect("Failed to open the window");
        unsafe { window.make_current() }
            .expect("Failed to set current context");

        let (window_x, window_y) = window.get_inner_size()
            .expect("Couldn't access window's size");

        //  Initialize OpenGL
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

        self.renderer.init(window_x as usize, window_y as usize);

        //  Camera position
        let mut camera = Camera::new(cgmath::Point3::<f32>::new(0.0, 0.0, 10.0));

        let mut active_object: Option<u32> = None;

        let textures_names = self.configuration.get_texture_names();
        // let textures = Visualization::load_textures(&mut factory, textures_names);

        let mut last_message_id: Option<String> = None;
        let mut objects: HashMap<u32, Object> = HashMap::<u32, Object>::new();
        let mut render_info: Vec<ObjectRenderInfo> = vec![];

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

            for event in window.poll_events() {
                use glutin::Event::*;

                match event {
                    Closed => {
                        break 'main
                    },
                    Resized(x, y) => {
                        self.renderer.resize(x as usize, y as usize);
                    },

                    MouseWheel(glutin::MouseScrollDelta::LineDelta(_, y), _) => {
                        camera.zoom(y);
                    },
                    MouseMoved(x, y) => {
                        let prev_pos = mouse_pos;

                        mouse_x = x as i32;
                        mouse_y = window.get_inner_size().unwrap().1 as i32 - y - 1;

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
                            let id = self.renderer.get_id((mouse_x as usize, mouse_y as usize));
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

            self.renderer.render(&render_info, camera_projection, active_object);

            window.swap_buffers()
                .expect("Failed to swap buffers");
        }

        let _ = self.link_core.send(None);
    }

    // fn empty_texture<F, R>(factory: &mut F, size: (usize, usize)) -> gfx::handle::ShaderResourceView<R, [f32; 4]>
    //     where F: gfx::Factory<R>,
    //           R: gfx::Resources {
    //
    //     use gfx::format::Rgba8;
    //     let (width, height) = size;
    //     let texture_mem: Vec<u8> = vec!(0; width * height * 4);
    //     let kind = gfx::texture::Kind::D2(width as u16, height as u16, gfx::texture::AaMode::Single);
    //     let (_, view) = factory.create_texture_immutable_u8::<Rgba8>(kind, &[texture_mem.as_slice()]).unwrap();
    //     view
    // }
    //
    // fn load_texture<F, R>(factory: &mut F, filename: &str) -> Option<gfx::handle::ShaderResourceView<R, [f32; 4]>>
    //     where F: gfx::Factory<R>,
    //           R: gfx::Resources {
    //
    //     use gfx::format::Rgba8;
    //     match image::open(filename) {
    //         Ok(img) => {
    //             let img = img.to_rgba();
    //             let (width, height) = img.dimensions();
    //             let kind = gfx::texture::Kind::D2(width as u16, height as u16, gfx::texture::AaMode::Single);
    //             let (_, view) = factory.create_texture_immutable_u8::<Rgba8>(kind, &[&img]).unwrap();
    //             Some(view)
    //         },
    //         Err(_)  => None
    //     }
    // }
    //
    // fn load_textures<F, R>(factory: &mut F, files: Vec<String>) -> HashMap<String, gfx::handle::ShaderResourceView<R, [f32; 4]>>
    //     where F: gfx::Factory<R>,
    //           R: gfx::Resources {
    //
    //     let mut result = HashMap::<String, gfx::handle::ShaderResourceView<R, [f32; 4]>>::new();
    //     for filename in files {
    //         match Visualization::load_texture(factory, &filename) {
    //             Some(tex) => {
    //                 result.insert(filename, tex);
    //             },
    //             None      => {},
    //         };
    //     }
    //     result
    // }
}
