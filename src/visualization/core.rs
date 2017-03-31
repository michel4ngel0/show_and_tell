use types::message::Message;
use visualization::configuration::Configuration;

use gfx;
use gfx_window_glutin;
use glutin;
use gfx::traits::FactoryExt;
use gfx::Device;
use std::sync::mpsc::Receiver;
use image;
use cgmath;
use cgmath::{Point3, Vector, Vector3, AffineMatrix3};

use std::collections::HashMap;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex Vertex {
        pos2d:  [f32; 2] = "pos2d",
        tex_uv: [f32; 2] = "tex_uv",
    }

    constant Locals {
        model            : [[f32; 4]; 4] = "u_Model",
        camera_projection: [[f32; 4]; 4] = "u_CameraProjection",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        model: gfx::Global<[[f32; 4]; 4]> = "u_Model",
        camera_projection: gfx::Global<[[f32; 4]; 4]> = "u_CameraProjection",
        locals: gfx::ConstantBuffer<Locals> = "Locals",
        tex: gfx::TextureSampler<[f32; 4]> = "t_Texture",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

const SQUARE_VERT: [Vertex; 4] = [
    Vertex { pos2d: [ -0.5, -0.5 ], tex_uv: [0.0, 1.0] },
    Vertex { pos2d: [ -0.5,  0.5 ], tex_uv: [0.0, 0.0] },
    Vertex { pos2d: [  0.5,  0.5 ], tex_uv: [1.0, 0.0] },
    Vertex { pos2d: [  0.5, -0.5 ], tex_uv: [1.0, 1.0] },
];

const SQUARE_IND: &'static [u16] = &[
    0, 2, 1,
    0, 2, 3,
];

pub struct Visualization {
    link: Receiver<Option<Message>>,
    publisher: String,
    configuration: Configuration,
}

impl Visualization {
    pub fn new(link: Receiver<Option<Message>>, publisher: String, config_file: String) -> Visualization {
        Visualization {
            link: link,
            publisher: publisher,
            configuration: Configuration::new(config_file),
        }
    }

    pub fn run(&self) {
        let window_name = format!("[{}]", self.publisher);
        let clear_color = [0.08, 0.08, 0.05, 1.0];

        let builder = glutin::WindowBuilder::new()
            .with_title(window_name)
            .with_dimensions(800, 600)
            .with_vsync();
        let (window, mut device, mut factory, main_color, mut main_depth) =
            gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);
        let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

        let pso = factory.create_pipeline_simple(
            include_bytes!("shader/triangle_150.glslv"),
            include_bytes!("shader/triangle_150.glslf"),
            pipe::new()
        ).unwrap();
        let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&SQUARE_VERT, SQUARE_IND);
        let sampler = factory.create_sampler_linear();
        let texture = Visualization::empty_texture(&mut factory);
        let mut data = pipe::Data {
            vbuf: vertex_buffer,
            model: cgmath::Matrix4::from_translation(Vector3::zero()).into(),
            camera_projection: cgmath::Matrix4::from_translation(Vector3::zero()).into(),
            locals: factory.create_constant_buffer(1),
            tex: (texture, sampler),
            out: main_color,
        };

        let mut direction = Vector3::new(0.0, 0.0, -0.1);
        let mut up = Vector3::new(0.0, 1.0, 0.0);
        let mut position = Point3::new(0.0, 0.0, 5.0);

        let textures_names = self.configuration.get_texture_names();
        let textures = Visualization::load_textures(&mut factory, textures_names);

        let mut recent_msgs = HashMap::<String, (Message, String, Vec<(f32, f32, f32)>)>::new();

        'main: loop {
            if let Ok(msg_option) = self.link.try_recv() {
                match msg_option {
                    Some(msg) => {
                        println!("(visualization) received message");
                        let topic_name = msg.topic.clone();
                        let (texture_name, positions) = self.configuration.get_render_info(&msg);
                        let _ = recent_msgs.insert(topic_name, (msg, texture_name, positions));
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

                    glutin::Event::Resized(_width, _height) => {
                        gfx_window_glutin::update_views(&window, &mut data.out, &mut main_depth);
                    },
                    _ => {},
                }
            }

            //  Draw the frame
            encoder.clear(&data.out, clear_color);

            let (window_x, window_y) = window.get_inner_size().unwrap();
            let aspect_ratio = (window_x as f32) / (window_y as f32);

            let camera_transformation: AffineMatrix3<f32> = cgmath::Transform::look_at(
                position,
                position + direction,
                up,
            );
            let proj = cgmath::perspective(cgmath::deg(75.0f32), aspect_ratio, 0.01, 1000.0);
            let camera_projection = (proj * camera_transformation.mat).into();

            let auxiliary_empty_texture = Visualization::empty_texture(&mut factory);
            for (_, &(ref msg, ref texture_name, ref positions)) in recent_msgs.iter() {
                let texture = match textures.get(texture_name) {
                    Some(tex) => tex,
                    None      => &auxiliary_empty_texture,
                };

                data.tex.0 = texture.clone();

                for &(x, y, _) in positions {
                    let model = cgmath::Matrix4::from_translation(Vector3::new(x, y, 0.0)).into();
                    let locals = Locals {
                        model: model,
                        camera_projection: camera_projection,
                    };
                    encoder.update_constant_buffer(&data.locals, &locals);
                    encoder.draw(&slice, &pso, &data);
                }
            }

            encoder.flush(&mut device);
            window.swap_buffers().unwrap();
            device.cleanup();
        }
    }

    fn empty_texture<F, R>(factory: &mut F) -> gfx::handle::ShaderResourceView<R, [f32; 4]>
        where F: gfx::Factory<R>,
              R: gfx::Resources {

        use gfx::format::Rgba8;
        let (width, height) = (64, 64);
        let texture_mem: Vec<u8> = vec!(0; width * height * 4);
        let kind = gfx::texture::Kind::D2(width as u16, height as u16, gfx::texture::AaMode::Single);
        let (_, view) = factory.create_texture_immutable_u8::<Rgba8>(kind, &[texture_mem.as_slice()]).unwrap();
        view
    }

    fn load_texture<F, R>(factory: &mut F, filename: &str) -> Option<gfx::handle::ShaderResourceView<R, [f32; 4]>>
        where F: gfx::Factory<R>,
              R: gfx::Resources {

        use gfx::format::Rgba8;
        match image::open(filename) {
            Ok(img) => {
                let img = img.to_rgba();
                let (width, height) = img.dimensions();
                let kind = gfx::texture::Kind::D2(width as u16, height as u16, gfx::texture::AaMode::Single);
                let (_, view) = factory.create_texture_immutable_u8::<Rgba8>(kind, &[&img]).unwrap();
                Some(view)
            },
            Err(_)  => None
        }
    }

    fn load_textures<F, R>(factory: &mut F, files: Vec<String>) -> HashMap<String, gfx::handle::ShaderResourceView<R, [f32; 4]>>
        where F: gfx::Factory<R>,
              R: gfx::Resources {

        let mut result = HashMap::<String, gfx::handle::ShaderResourceView<R, [f32; 4]>>::new();
        for filename in files {
            match Visualization::load_texture(factory, &filename) {
                Some(tex) => {
                    result.insert(filename, tex);
                },
                None      => {},
            };
        }
        result
    }
}
