use gl;
use gl::types::*;

use cgmath;
use image;

use std::ptr;
use std::str;
use std::ffi::CString;
use std::mem;
use std::collections::HashMap;
use std::f64::consts::PI;

use types::{Geometry, ObjectRenderInfo};

const SQUARE_VERTICES: &'static [GLfloat] = &[
    -0.5, -0.5, -0.5, 0.0, 0.0,
    -0.5,  0.5, -0.5, 0.0, 1.0,
     0.5,  0.5, -0.5, 1.0, 1.0,
     0.5, -0.5, -0.5, 1.0, 0.0,
];

const CUBE_VERTICES: &'static [GLfloat] = &[
     -0.45, -0.45, -0.45, 0.0, 0.0,
     -0.45,  0.45, -0.45, 0.0, 1.0,
      0.45,  0.45, -0.45, 1.0, 1.0,
      0.45, -0.45, -0.45, 1.0, 0.0,
     -0.45, -0.45,  0.45, 1.0, 1.0,
     -0.45,  0.45,  0.45, 1.0, 0.0,
      0.45,  0.45,  0.45, 0.0, 0.0,
      0.45, -0.45,  0.45, 0.0, 1.0,
];

const SQUARE_INDICES: &'static [GLuint] = &[
    1, 0, 3,
    1, 2, 3,
];

const CUBE_INDICES: &'static [GLuint] = &[
    1, 0, 3,
    1, 2, 3,
    4, 0, 3,
    4, 7, 3,
    3, 7, 6,
    3, 2, 6,
    1, 2, 6,
    1, 5, 6,
    1, 0, 4,
    1, 5, 4,
    4, 5, 6,
    4, 7, 6,
];

pub struct Renderer {
    x: usize,
    y: usize,

    framebuffer:   Option<GLuint>,
    texture_color: Option<GLuint>,
    texture_id:    Option<GLuint>,
    rbo_depth:     Option<GLuint>,

    texture_font: Option<GLuint>,

    textures: HashMap<String, GLuint>,

    square_v_buffer: Option<GLuint>,
    square_i_buffer: Option<GLuint>,

    cube_v_buffer: Option<GLuint>,
    cube_i_buffer: Option<GLuint>,

    scene_v_shader: Option<GLuint>,
    scene_f_shader: Option<GLuint>,
    scene_program:  Option<GLuint>,
    quad_v_shader:  Option<GLuint>,
    quad_f_shader:  Option<GLuint>,
    quad_program:   Option<GLuint>,
    glyph_v_shader:  Option<GLuint>,
    glyph_f_shader:  Option<GLuint>,
    glyph_program:   Option<GLuint>,

    used_model: Option<Geometry>,
}

impl Renderer {
    pub fn new(x: usize, y: usize, textures: Vec<String>) -> Renderer {
        let mut new_renderer = Renderer {
            x: 0,
            y: 0,

            framebuffer:   None,
            texture_color: None,
            texture_id:    None,
            rbo_depth:     None,

            texture_font: None,

            textures: HashMap::<String, GLuint>::new(),

            square_v_buffer: None,
            square_i_buffer: None,

            cube_v_buffer: None,
            cube_i_buffer: None,

            scene_v_shader: None,
            scene_f_shader: None,
            scene_program:  None,
            quad_v_shader:  None,
            quad_f_shader:  None,
            quad_program:   None,
            glyph_v_shader: None,
            glyph_f_shader: None,
            glyph_program:  None,

            used_model: None,
        };
        new_renderer.init(x, y, textures);
        new_renderer
    }

    fn init(&mut self, x: usize, y: usize, textures: Vec<String>) {
        self.gen_vertex_index_buffers();
        self.resize(x, y);
        self.compile_shaders();
        self.load_textures(textures);
        self.load_font();
    }

    fn bind_square(&mut self) {
        unsafe {
            match self.used_model {
                Some(Geometry::Square) => {},
                _                    => {
                    gl::BindBuffer(gl::ARRAY_BUFFER, self.square_v_buffer.unwrap());
                    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.square_i_buffer.unwrap());
                    self.used_model = Some(Geometry::Square);
                },
            };
        }
    }

    fn bind_cube(&mut self) {
        unsafe {
            match self.used_model {
                Some(Geometry::Cube) => {},
                _                    => {
                    gl::BindBuffer(gl::ARRAY_BUFFER, self.cube_v_buffer.unwrap());
                    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.cube_i_buffer.unwrap());
                    self.used_model = Some(Geometry::Cube);
                },
            };
        }
    }

    pub fn render(
            &mut self,
            objects: &Vec<ObjectRenderInfo>,
            camera_projection: cgmath::Matrix4<f32>,
            active_object: Option<u32>,
            strings: Vec<String>,
            phi: f64) {

        let framebuffer = self.framebuffer
            .expect("Cannot render, framebuffer has not been initialized");

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
        }

        //  Draw to the framebuffer
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
            check_framebuffer_status();

            gl::DepthFunc(gl::LEQUAL);

            gl::ClearDepth(1.0);
            gl::Clear(gl::DEPTH_BUFFER_BIT);

            gl::DrawBuffer(gl::COLOR_ATTACHMENT0);
            gl::ClearColor(0.9, 0.9, 0.85, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::DrawBuffer(gl::COLOR_ATTACHMENT1);
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            let buffers: [GLenum; 2] = [gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1];
            gl::DrawBuffers(2, &(buffers[0]) as *const GLenum);

            if let Some(program) = self.scene_program {
                gl::UseProgram(program);
                gl::BindFragDataLocation(program, 0, CString::new("out_color").unwrap().as_ptr());
                gl::BindFragDataLocation(program, 1, CString::new("out_index").unwrap().as_ptr());

                let pos_attribute = gl::GetAttribLocation(
                    program,
                    CString::new("v_pos_xyz").unwrap().as_ptr()
                );
                let tex_attribute = gl::GetAttribLocation(
                    program,
                    CString::new("v_tex_uv").unwrap().as_ptr()
                );
                let tex_bound_uniform_loc = gl::GetUniformLocation(
                    program,
                    CString::new("u_texture_bound").unwrap().as_ptr()
                );
                let model_uniform_loc = gl::GetUniformLocation(
                    program,
                    CString::new("u_model").unwrap().as_ptr()
                );
                let cam_proj_uniform_loc = gl::GetUniformLocation(
                    program,
                    CString::new("u_camera_projection").unwrap().as_ptr()
                );
                let tex_uniform_loc = gl::GetUniformLocation(
                    program,
                    CString::new("u_texture").unwrap().as_ptr()
                );
                let color_uniform_loc = gl::GetUniformLocation(
                    program,
                    CString::new("u_color").unwrap().as_ptr()
                );
                let id_uniform_loc = gl::GetUniformLocation(
                    program,
                    CString::new("u_id").unwrap().as_ptr()
                );
                let highlight_uniform_loc = gl::GetUniformLocation(
                    program,
                    CString::new("u_selection_highlight").unwrap().as_ptr()
                );

                gl::UniformMatrix4fv(
                    cam_proj_uniform_loc,
                    1,
                    gl::FALSE,
                    mem::transmute(&camera_projection)
                );

                for object in objects {
                    match object.model {
                        Geometry::Square => {
                            self.bind_square();
                        },
                        Geometry::Cube   => {
                            self.bind_cube();
                        }
                    }

                    if let Some(id) = active_object {
                        if id == object.id {
                            let phi = 8.0 * phi;
                            gl::Uniform3f(
                                highlight_uniform_loc,
                                phi.sin() as f32,
                                (phi + 2.0 * PI / 3.0).sin() as f32,
                                (phi + 4.0 * PI / 3.0).sin() as f32
                            );
                        } else {
                            gl::Uniform3f(highlight_uniform_loc, 0.0, 0.0, 0.0);
                        }
                    } else {
                        gl::Uniform3f(highlight_uniform_loc, 0.0, 0.0, 0.0);
                    }

                    let model_matrix = cgmath::Matrix4::from_translation(cgmath::Vector3::<f32>::new(
                        object.position.0,
                        object.position.1,
                        object.position.2,
                    ));

                    let texture_handle = match self.textures.get(&object.texture_name) {
                        Some(handle) => handle.clone(),
                        None         => 0 as GLuint,
                    };
                    gl::ActiveTexture(gl::TEXTURE0);
                    gl::BindTexture(gl::TEXTURE_2D, texture_handle);
                    gl::Uniform1i(tex_uniform_loc, 0);

                    gl::Uniform1i(tex_bound_uniform_loc, texture_handle as i32);

                    {
                        let (r, g, b) = object.color;
                        gl::Uniform3f(color_uniform_loc, r, g, b);
                    }

                    gl::UniformMatrix4fv(
                        model_uniform_loc,
                        1,
                        gl::FALSE,
                        mem::transmute(&model_matrix)
                    );
                    gl::Uniform1ui(id_uniform_loc, object.id);

                    gl::EnableVertexAttribArray(pos_attribute as GLuint);
                    gl::EnableVertexAttribArray(tex_attribute as GLuint);
                    gl::VertexAttribPointer(
                        pos_attribute as GLuint,
                        3,
                        gl::FLOAT,
                        gl::FALSE as GLboolean,
                        (5 * mem::size_of::<GLfloat>()) as i32,
                        ptr::null()
                    );
                    gl::VertexAttribPointer(
                        tex_attribute as GLuint,
                        2,
                        gl::FLOAT,
                        gl::FALSE as GLboolean,
                        (5 * mem::size_of::<GLfloat>()) as i32,
                        mem::transmute(3 * mem::size_of::<GLfloat>())
                    );

                    match object.model {
                        Geometry::Square => {
                            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
                        },
                        Geometry::Cube   => {
                            gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_INT, ptr::null());
                        }
                    }
                };

                gl::DisableVertexAttribArray(pos_attribute as GLuint);
                gl::DisableVertexAttribArray(tex_attribute as GLuint);
            }

            gl::Flush();
        }

        check_gl_error("rendering to framebuffer");

        //  Draw strings
        unsafe {
            const FONT_HEIGHT: f32 = 16.0;
            const FONT_WIDTH: f32 = 8.0;
            let screen_width = self.x as f32;
            let screen_height = self.y as f32;

            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer.unwrap());

            gl::DepthFunc(gl::ALWAYS);
            gl::BlendFuncSeparate(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA, gl::ZERO, gl::ONE);

            gl::DrawBuffer(gl::COLOR_ATTACHMENT0);

            check_gl_error("drawbuffers");

            if let Some(program) = self.glyph_program {
                gl::UseProgram(program);
                gl::BindFragDataLocation(program, 0, CString::new("out_color").unwrap().as_ptr());

                let pos_attribute = gl::GetAttribLocation(
                    program,
                    CString::new("v_pos_xy").unwrap().as_ptr()
                );
                let tex_attribute = gl::GetAttribLocation(
                    program,
                    CString::new("v_tex_uv").unwrap().as_ptr()
                );

                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, self.texture_font.unwrap());
                let texture_uniform_loc = gl::GetUniformLocation(
                    program,
                    CString::new("u_font_bitmap").unwrap().as_ptr()
                );
                gl::Uniform1i(texture_uniform_loc, 0);
                check_gl_error("texture");

                let transform_uniform_loc = gl::GetUniformLocation(
                    program,
                    CString::new("u_transform").unwrap().as_ptr()
                );
                let tex_u_uniform_loc = gl::GetUniformLocation(
                    program,
                    CString::new("tex_u").unwrap().as_ptr()
                );
                let tex_v_uniform_loc = gl::GetUniformLocation(
                    program,
                    CString::new("tex_v").unwrap().as_ptr()
                );

                self.bind_square();

                gl::EnableVertexAttribArray(pos_attribute as GLuint);
                gl::EnableVertexAttribArray(tex_attribute as GLuint);
                gl::VertexAttribPointer(
                    pos_attribute as GLuint,
                    2,
                    gl::FLOAT,
                    gl::FALSE as GLboolean,
                    (5 * mem::size_of::<GLfloat>()) as i32,
                    ptr::null()
                );
                gl::VertexAttribPointer(
                    tex_attribute as GLuint,
                    2,
                    gl::FLOAT,
                    gl::FALSE as GLboolean,
                    (5 * mem::size_of::<GLfloat>()) as i32,
                    mem::transmute(3 * mem::size_of::<GLfloat>())
                );

                let mut x_pos = 0.0;
                let mut y_pos = screen_height;
                for text in strings {
                    //  New Line
                    y_pos -= FONT_HEIGHT;
                    x_pos = 0.0;

                    let ascii = string_to_renderable(text);
                    for byte in ascii {
                        if byte == 10 { /* /n */
                            y_pos -= FONT_HEIGHT;
                            x_pos = 0.0;
                        } else if byte == 9 { /* /t */
                            x_pos += 4.0;
                        } else {
                            let (tex_u, tex_v) = get_glyph_position(byte);
                            gl::Uniform1f(tex_u_uniform_loc, tex_u);
                            gl::Uniform1f(tex_v_uniform_loc, tex_v);

                            let transform = cgmath::Matrix3::<f32>::new(
                                2.0 * FONT_WIDTH / screen_width, 0.0, 0.0,
                                0.0, 2.0 * FONT_HEIGHT / screen_height, 0.0,
                                (2.0 * x_pos + FONT_WIDTH - screen_width) / screen_width, (2.0 * y_pos + FONT_HEIGHT - screen_height) / screen_height, 1.0
                            );
                            gl::UniformMatrix3fv(
                                transform_uniform_loc,
                                1,
                                gl::FALSE,
                                mem::transmute(&transform)
                            );

                            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
                            x_pos += FONT_WIDTH;
                        }
                    }
                }

                gl::DisableVertexAttribArray(pos_attribute as GLuint);
                gl::DisableVertexAttribArray(tex_attribute as GLuint);
            }

            gl::Flush();
        }

        check_gl_error("drawing strings");

        //  Draw on the screen
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            gl::Viewport(0, 0, self.x as i32, self.y as i32);

            gl::DepthFunc(gl::ALWAYS);

            gl::ClearColor(1.0, 0.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            if let Some(program) = self.quad_program {
                gl::UseProgram(program);
                gl::BindFragDataLocation(program, 0, CString::new("out_color").unwrap().as_ptr());

                let pos_attribute = gl::GetAttribLocation(
                    program,
                    CString::new("v_pos_xy").unwrap().as_ptr()
                );
                let tex_attribute = gl::GetAttribLocation(
                    program,
                    CString::new("v_tex_uv").unwrap().as_ptr()
                );

                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, self.texture_color.unwrap());
                let texture_uniform_loc = gl::GetUniformLocation(
                    program,
                    CString::new("u_rendered_scene").unwrap().as_ptr()
                );
                gl::Uniform1i(texture_uniform_loc, 0);

                self.bind_square();

                gl::EnableVertexAttribArray(pos_attribute as GLuint);
                gl::EnableVertexAttribArray(tex_attribute as GLuint);
                gl::VertexAttribPointer(
                    pos_attribute as GLuint,
                    2,
                    gl::FLOAT,
                    gl::FALSE as GLboolean,
                    (5 * mem::size_of::<GLfloat>()) as i32,
                    ptr::null()
                );
                gl::VertexAttribPointer(
                    tex_attribute as GLuint,
                    2,
                    gl::FLOAT,
                    gl::FALSE as GLboolean,
                    (5 * mem::size_of::<GLfloat>()) as i32,
                    mem::transmute(3 * mem::size_of::<GLfloat>())
                );

                gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());

                gl::DisableVertexAttribArray(pos_attribute as GLuint);
                gl::DisableVertexAttribArray(tex_attribute as GLuint);
            }

            gl::Flush();
        }

        check_gl_error("rendering to window");
    }

    pub fn get_id(&self, size: (usize, usize)) -> Option<u32> {
        match self.framebuffer {
            Some(framebuffer) => {
                let (x, y) = size;
                if x >= self.x || y >= self.y {
                    return None;
                }

                let mut id: u32 = u32::max_value();
                unsafe {
                    gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
                    check_framebuffer_status();

                    gl::ReadBuffer(gl::COLOR_ATTACHMENT1);

                    gl::ReadPixels(
                        x as i32,
                        y as i32,
                        1,
                        1,
                        gl::RED_INTEGER,
                        gl::UNSIGNED_INT,
                        mem::transmute(&mut id)
                    );

                    gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                }

                check_gl_error("reading object id");

                if id > 1000000000 { None } else { Some(id) }
            },
            None => None,
        }
    }

    pub fn resize(&mut self, x: usize, y: usize) {
        if x == 0 || y == 0 {
            return;
        }

        self.drop_framebuffer_with_attachments();

        self.x = x;
        self.y = y;
        let x = x as i32;
        let y = y as i32;

        let mut tex_color_handle         = 0;
        let mut tex_id_handle: GLuint    = 0;
        let mut rbo_depth_handle: GLuint = 0;
        let mut framebuff_handle: GLuint = 0;

        unsafe {
            gl::GenFramebuffers(1, &mut framebuff_handle as *mut GLuint);
            gl::GenTextures(1, &mut tex_color_handle as *mut GLuint);
            gl::GenTextures(1, &mut tex_id_handle as *mut GLuint);
            gl::GenRenderbuffers(1, &mut rbo_depth_handle as *mut GLuint);
        }

        check_gl_error("generating framebuffer");

        if tex_id_handle == 0 || rbo_depth_handle == 0 || framebuff_handle == 0 {
            self.drop_framebuffer_with_attachments();
            return;
        }

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);

            gl::BindTexture(gl::TEXTURE_2D, tex_color_handle);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, x, y, 0, gl::RGBA, gl::UNSIGNED_BYTE, ptr::null());

            gl::BindTexture(gl::TEXTURE_2D, tex_id_handle);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::R32UI as i32, x, y, 0, gl::RED_INTEGER, gl::UNSIGNED_INT, ptr::null());

            gl::BindRenderbuffer(gl::RENDERBUFFER, rbo_depth_handle);
            gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, x, y);
        }

        check_gl_error("creating textures");

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuff_handle);

            gl::BindTexture(gl::TEXTURE_2D, tex_color_handle);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, tex_color_handle, 0);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::BindTexture(gl::TEXTURE_2D, tex_id_handle);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, gl::TEXTURE_2D, tex_id_handle, 0);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::BindRenderbuffer(gl::RENDERBUFFER, rbo_depth_handle);
            gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, rbo_depth_handle);
        }

        check_gl_error("binding render targets");

        check_framebuffer_status();

        self.texture_color = Some(tex_color_handle);
        self.texture_id    = Some(tex_id_handle);
        self.rbo_depth     = Some(rbo_depth_handle);
        self.framebuffer   = Some(framebuff_handle);
    }

    pub fn gen_vertex_index_buffers(&mut self) {
        self.drop_vertex_index_buffers();

        let mut square_v_handle: GLuint = 0;
        let mut square_i_handle: GLuint = 0;
        let mut cube_v_handle: GLuint   = 0;
        let mut cube_i_handle: GLuint   = 0;

        unsafe {
            gl::GenBuffers(1, &mut square_v_handle as *mut GLuint);
            gl::GenBuffers(1, &mut square_i_handle as *mut GLuint);
            gl::GenBuffers(1, &mut cube_v_handle as *mut GLuint);
            gl::GenBuffers(1, &mut cube_i_handle as *mut GLuint);
        }

        check_gl_error("generating buffers");

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, square_v_handle);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (SQUARE_VERTICES.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                mem::transmute(&SQUARE_VERTICES[0]),
                gl::STATIC_DRAW
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, square_i_handle);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (SQUARE_INDICES.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
                mem::transmute(&SQUARE_INDICES[0]),
                gl::STATIC_DRAW
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, cube_v_handle);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (CUBE_VERTICES.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                mem::transmute(&CUBE_VERTICES[0]),
                gl::STATIC_DRAW
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, cube_i_handle);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (CUBE_INDICES.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
                mem::transmute(&CUBE_INDICES[0]),
                gl::STATIC_DRAW
            );
        }

        check_gl_error("initializing buffers");

        self.square_v_buffer = Some(square_v_handle);
        self.square_i_buffer = Some(square_i_handle);
        self.cube_v_buffer = Some(cube_v_handle);
        self.cube_i_buffer = Some(cube_i_handle);
    }

    fn compile_shaders(&mut self) {
        self.drop_shaders();

        self.scene_v_shader = Some(compile_shader(include_bytes!("shader/render_scene.glslv"), gl::VERTEX_SHADER));
        self.scene_f_shader = Some(compile_shader(include_bytes!("shader/render_scene.glslf"), gl::FRAGMENT_SHADER));
        self.scene_program  = Some(link_simple_program(self.scene_v_shader.unwrap(), self.scene_f_shader.unwrap()));

        check_gl_error("compiling shaders");

        self.quad_v_shader = Some(compile_shader(include_bytes!("shader/quad.glslv"), gl::VERTEX_SHADER));
        self.quad_f_shader = Some(compile_shader(include_bytes!("shader/quad.glslf"), gl::FRAGMENT_SHADER));
        self.quad_program  = Some(link_simple_program(self.quad_v_shader.unwrap(), self.quad_f_shader.unwrap()));

        check_gl_error("compiling shaders");

        self.glyph_v_shader = Some(compile_shader(include_bytes!("shader/glyph.glslv"), gl::VERTEX_SHADER));
        self.glyph_f_shader = Some(compile_shader(include_bytes!("shader/glyph.glslf"), gl::FRAGMENT_SHADER));
        self.glyph_program  = Some(link_simple_program(self.glyph_v_shader.unwrap(), self.glyph_f_shader.unwrap()));

        check_gl_error("compiling shaders");
    }

    fn load_textures(&mut self, files: Vec<String>) {
        self.drop_textures();

        for file in files {
            match load_texture_from_file(&file) {
                Some(handle) => { self.textures.insert(file, handle); },
                None         => { println!("Failed to open file {}", file); },
            };
        }
    }

    fn load_font(&mut self) {
        self.drop_font();

        let texture_handle = load_texture_from_file(&String::from("resources/ascii.png"))
            .expect("Could not load the font bitmap");

        self.texture_font = Some(texture_handle);

        check_gl_error("loading font bitmap");
    }

    fn drop_font(&mut self) {
        unsafe {
            if let Some(texture_handle) = self.texture_font {
                gl::DeleteTextures(1, &texture_handle as *const GLuint);
            }
        }
        self.texture_font = None;

        check_gl_error("dropping font bitmap");
    }

    fn drop_textures(&mut self) {
        unsafe {
            for (_, texture_handle) in &self.textures {
                gl::DeleteTextures(1, texture_handle as *const GLuint);
            }
        }
        self.textures.clear();

        check_gl_error("dropping textures");
    }

    fn drop_shaders(&mut self) {
        unsafe {
            if let Some(program) = self.scene_program {
                gl::DeleteProgram(program);
                self.scene_program = None;
            }
            if let Some(program) = self.quad_program {
                gl::DeleteProgram(program);
                self.quad_program = None;
            }
            if let Some(program) = self.glyph_program {
                gl::DeleteProgram(program);
                self.glyph_program = None;
            }
            if let Some(shader) = self.scene_v_shader {
                gl::DeleteShader(shader);
                self.scene_v_shader = None;
            }
            if let Some(shader) = self.scene_f_shader {
                gl::DeleteShader(shader);
                self.scene_f_shader = None;
            }
            if let Some(shader) = self.quad_v_shader {
                gl::DeleteShader(shader);
                self.quad_v_shader = None;
            }
            if let Some(shader) = self.quad_f_shader {
                gl::DeleteShader(shader);
                self.quad_f_shader = None;
            }
            if let Some(shader) = self.glyph_v_shader {
                gl::DeleteShader(shader);
                self.glyph_v_shader = None;
            }
            if let Some(shader) = self.glyph_f_shader {
                gl::DeleteShader(shader);
                self.glyph_f_shader = None;
            }
        }

        check_gl_error("dropping shaders");
    }

    fn drop_framebuffer_with_attachments(&mut self) {
        if let Some(texture) = self.texture_color {
            unsafe {
                gl::DeleteTextures(1, &texture as *const GLuint);
            }
            self.texture_color = None;
        }
        if let Some(texture) = self.texture_id {
            unsafe {
                gl::DeleteTextures(1, &texture as *const GLuint);
            }
            self.texture_id = None;
        }
        if let Some(rbo) = self.rbo_depth {
            unsafe {
                gl::DeleteRenderbuffers(1, &rbo as *const GLuint);
            }
            self.rbo_depth = None;
        }
        if let Some(framebuffer) = self.framebuffer {
            unsafe {
                gl::DeleteFramebuffers(1, &framebuffer as *const GLuint);
            }
            self.framebuffer = None;
        }

        check_gl_error("dropping framebuffer");
    }

    fn drop_vertex_index_buffers(&mut self) {
        unsafe {
            if let Some(mut buffer) = self.square_v_buffer {
                gl::DeleteBuffers(1, &mut buffer as *mut GLuint);
            }
            self.square_v_buffer = None;

            if let Some(mut buffer) = self.square_i_buffer {
                gl::DeleteBuffers(1, &mut buffer as *mut GLuint);
            }
            self.square_i_buffer = None;

            if let Some(mut buffer) = self.cube_v_buffer {
                gl::DeleteBuffers(1, &mut buffer as *mut GLuint);
            }
            self.cube_v_buffer = None;

            if let Some(mut buffer) = self.cube_i_buffer {
                gl::DeleteBuffers(1, &mut buffer as *mut GLuint);
            }
            self.cube_i_buffer = None;
        }

        check_gl_error("dropping buffers");
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.drop_framebuffer_with_attachments();
        self.drop_vertex_index_buffers();
        self.drop_shaders();
        self.drop_textures();
        self.drop_font();

        check_gl_error("dropping resources");
    }
}

fn load_texture_from_file(file: &String) -> Option<GLuint> {
    match image::open(file) {
        Ok(bitmap) => {
            let bitmap = bitmap.to_rgba();
            let (width, height) = bitmap.dimensions();

            let mut texture_handle: GLuint = 0;

            unsafe {
                gl::GenTextures(1, &mut texture_handle as *mut GLuint);

                gl::ActiveTexture(gl::TEXTURE0);

                gl::BindTexture(gl::TEXTURE_2D, texture_handle);
                gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, width as i32, height as i32, 0, gl::RGBA, gl::UNSIGNED_BYTE, mem::transmute(bitmap.into_raw().as_ptr()));
                gl::GenerateMipmap(gl::TEXTURE_2D);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            }

            check_gl_error("loading textures");

            Some(texture_handle)
        },
        Err(_)  => None
    }
}

//  Copy-pasted from gl-rs package examples
fn compile_shader(src: &[u8], ty: GLenum) -> GLuint {
    let shader;
    unsafe {
        shader = gl::CreateShader(ty);
        // Attempt to compile the shader
        let c_str = CString::new(src).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            panic!("{}", str::from_utf8(&buf).ok().expect("ShaderInfoLog not valid utf8"));
        }
    }
    shader
}

//  Same as above
fn link_simple_program(vs: GLuint, fs: GLuint) -> GLuint { unsafe {
    let program = gl::CreateProgram();
    gl::AttachShader(program, vs);
    gl::AttachShader(program, fs);
    gl::LinkProgram(program);
    // Get the link status
    let mut status = gl::FALSE as GLint;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

    // Fail on error
    if status != (gl::TRUE as GLint) {
        let mut len: GLint = 0;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf = Vec::with_capacity(len as usize);
        buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
        gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
        panic!("{}", str::from_utf8(&buf).ok().expect("ProgramInfoLog not valid utf8"));
    }
    program
} }

fn string_to_renderable(text: String) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::<u8>::new();

    for mut byte in text.into_bytes() {
        if byte > 127 { continue; }
        result.push(byte);
    }

    result
}

fn get_glyph_position(mut byte: u8) -> (f32, f32) {
    byte = if byte < 32 { 0 } else { byte - 32 };
    let (x, y) = (byte % 16, byte / 16);
    let (x, y) = (x as f32 / 16.0, y as f32 / 6.0);
    // let y = 1.0 - y - (1.0 / 6.0);
    (x, y)
}

fn check_framebuffer_status() {
    unsafe {
        let status: u32 = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);

        if status != gl::FRAMEBUFFER_COMPLETE {
            let error_msg = match status {
                gl::FRAMEBUFFER_UNDEFINED =>
                    "GL_FRAMEBUFFER_UNDEFINED",
                gl::FRAMEBUFFER_INCOMPLETE_ATTACHMENT =>
                    "GL_FRAMEBUFFER_INCOMPLETE_ATTACHMENT",
                gl::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT =>
                    "GL_FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT",
                gl::FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER =>
                    "GL_FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER",
                gl::FRAMEBUFFER_INCOMPLETE_READ_BUFFER =>
                    "GL_FRAMEBUFFER_INCOMPLETE_READ_BUFFER",
                gl::FRAMEBUFFER_UNSUPPORTED =>
                    "GL_FRAMEBUFFER_UNSUPPORTED ",
                gl::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE =>
                    "GL_FRAMEBUFFER_INCOMPLETE_MULTISAMPLE",
                gl::FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS =>
                    "GL_FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS",
                gl::FRAMEBUFFER_COMPLETE =>
                    "GL_FRAMEBUFFER_COMPLETE",
                _ => "unknown"
            };
            println!("Framebuffer error: {}", error_msg);
        }
    }

    check_gl_error("checking framebuffer status");
}

fn check_gl_error(when: &str) {
    unsafe {
        let error = gl::GetError();

        if error != 0 {
            let error_msg = match error {
                gl::INVALID_ENUM => "GL_INVALID_ENUM",
                gl::INVALID_VALUE => "GL_INVALID_VALUE",
                gl::INVALID_OPERATION => "GL_INVALID_OPERATION",
                gl::STACK_OVERFLOW => "GL_STACK_OVERFLOW",
                gl::STACK_UNDERFLOW => "GL_STACK_UNDERVLOW",
                gl::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
                gl::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
                gl::CONTEXT_LOST => "GL_CONTEXT_LOST",
                _ => "unknown",
            };

            println!("OpenGL error: {} ({})", error_msg, when);
            // panic!();
        }
    }
}
