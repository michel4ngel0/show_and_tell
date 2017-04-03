use types::message::MessageIn;
use types::ObjectRenderInfo;
use visualization::configuration::Configuration;
use visualization::render::Renderer;

use glutin;
use gl;
use image;
use cgmath;
use cgmath::{Point3, Vector3, AffineMatrix3};

use std::sync::mpsc::Receiver;

pub struct Visualization {
    link: Receiver<Option<MessageIn>>,
    publisher: String,
    configuration: Configuration,
    renderer: Renderer,
}

impl Visualization {
    pub fn new(link: Receiver<Option<MessageIn>>, publisher: String, config_file: String) -> Visualization {
        Visualization {
            link: link,
            publisher: publisher,
            configuration: Configuration::new(config_file),
            renderer: Renderer::new(),
        }
    }

    pub fn run(&mut self) {
        let window_name = format!("[{}]", self.publisher);

        let window = glutin::WindowBuilder::new()
            .with_title(window_name)
            .with_dimensions(800, 600)
            // .with_vsync()
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

        let mut direction = Vector3::new(0.0, 0.0, -0.1);
        let mut up = Vector3::new(0.0, 1.0, 0.0);
        let mut position = Point3::new(0.0, 0.0, 10.0);

        let textures_names = self.configuration.get_texture_names();
        // let textures = Visualization::load_textures(&mut factory, textures_names);

        let mut recent_msg: Option<(MessageIn, Vec<ObjectRenderInfo>)> = None;

        let mut mouse_x = 0;
        let mut mouse_y = 0;

        'main: loop {
            if let Ok(msg_option) = self.link.try_recv() {
                match msg_option {
                    Some(msg) => {
                        let render_info = self.configuration.get_render_info(&msg);
                        recent_msg = Some((msg, render_info));
                    },
                    None      => {
                        println!("(visualization) terminating");
                        break 'main;
                    },
                };
            }

            for event in window.poll_events() {
                match event {
                    glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                    glutin::Event::Closed => break 'main,

                    glutin::Event::MouseWheel(glutin::MouseScrollDelta::LineDelta(_, y), _) =>
                        position = position + if y > 0.0 { direction } else { -direction },

                    glutin::Event::Resized(x, y) => {
                        self.renderer.resize(x as usize, y as usize);
                    },

                    glutin::Event::MouseMoved(x, y) => {
                        mouse_x = x as i32;
                        mouse_y = window.get_inner_size().unwrap().1 as i32 - y - 1;
                    },

                    glutin::Event::MouseInput(glutin::ElementState::Pressed, glutin::MouseButton::Left) => {
                        let id = self.renderer.get_id((mouse_x as usize, mouse_y as usize));

                        println!("Click! ({}, {}): id {:?}", mouse_x, mouse_y, id);
                    },

                    _ => {},
                }
            }

            //  Calculate uniform tranformations
            let (window_x, window_y) = window.get_inner_size().unwrap();
            let aspect_ratio = (window_x as f32) / (window_y as f32);

            let camera_transformation: AffineMatrix3<f32> = cgmath::Transform::look_at(
                position,
                position + direction,
                up,
            );
            let proj = cgmath::perspective(cgmath::deg(75.0f32), aspect_ratio, 0.01, 1000.0);
            let camera_projection = (proj * camera_transformation.mat).into();

            if let Some((_, ref render_info)) = recent_msg {
                self.renderer.render(&render_info, camera_projection);
            }

            window.swap_buffers()
                .expect("Failed to swap buffers");
        }
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
