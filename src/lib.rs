#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate cgmath;
extern crate rustc_serialize;
extern crate image;
extern crate regex;

mod server;
mod visualization;
mod types;

pub use server::core::Server as Server;
