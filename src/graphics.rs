use std::path::Path;

use glium;
use glium_sdl2::{DisplayBuild, SDL2Facade};
use image;
use sdl2;

use crate::config::ApplicationGDXConfig;

pub mod animation;
pub mod shape;
pub mod sprite;
pub mod text;
pub mod texture;

pub struct Graphics {
    display: SDL2Facade,
}

impl Graphics {
    pub fn new(config: &ApplicationGDXConfig, sdl_context: &sdl2::Sdl) -> Self {
        let video_subsystem = sdl_context.video().unwrap();

        video_subsystem.gl_attr().set_context_version(3, 3);
        video_subsystem.gl_attr().set_context_profile(sdl2::video::GLProfile::Core);

        let screen_size = config.screen_size();
        let mut window_builder = video_subsystem.window(config.title(), screen_size.0, screen_size.1);
        if config.resizable() {
            window_builder.resizable();
        }
        let display = window_builder
            .build_glium()
            .expect("Could not build glium window.");

        let swap_interval = if config.vsync() { 1 } else { 0 };
        video_subsystem.gl_set_swap_interval(swap_interval)
            .expect("Could not set OpenGL swap interval.");

        Self {
            display,
        }
    }

    pub fn set_size(&mut self, width: u32, height: u32) {
        self.display.window_mut().set_size(width, height)
            .unwrap();
    }

    pub fn display(&self) -> &SDL2Facade {
        &self.display
    }

    pub fn screen_size(&self) -> (u32, u32) {
        self.display.get_framebuffer_dimensions()
    }

    pub fn load_texture<P: AsRef<Path>>(&self, path: P, reversed: bool) -> glium::Texture2d {
        let image = image::open(path).unwrap().to_rgba();
        let image_dimensions = image.dimensions();
        let image = if reversed {
            glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions)
        } else {
            glium::texture::RawImage2d::from_raw_rgba(image.into_raw(), image_dimensions)
        };
        glium::Texture2d::new(&self.display, image).unwrap()
    }

    fn draw(&self) {
    }
}
