pub use glium::{Surface, Texture2d};
use moving_average::MovingAverage;

pub use crate::app::AppGDX;
pub use crate::config::ApplicationGDXConfig;
pub use crate::input::{Axis, Button, Input, KeyCode, MouseButton};

use std::time::{
    Duration,
    Instant,
};
use std::thread;

use crate::graphics::Graphics;
use crate::input::ElementState;
use crate::time::Time;

mod app;
mod config;
pub mod graphics;
mod input;
mod time;

pub struct GDXLauncher<T: AppGDX> {
    frame_duration: Duration,
    main: ApplicationGDX,
    app: T,
}

impl<T: AppGDX> GDXLauncher<T> {
    pub fn new(config: ApplicationGDXConfig) -> Self {
        let frame_time_ns = (1_000_000_000.0 / config.fps() as f64) as u64;
        let frame_duration = Duration::from_nanos(frame_time_ns);

        let main = ApplicationGDX::new(&config);
        let app = T::new(&main);

        GDXLauncher {
            frame_duration,
            main,
            app,
        }
    }

    pub fn run(mut self) {
        let mut window_closed = false;
        let mut win_size = self.main.graphics.screen_size();
        let mut resized: Option<(u32, u32)> = None;

        while !window_closed && !self.main.should_exit() {
            let start_time = Instant::now();
            self.main.time.update();
            self.main.delta_times.add(self.main.time.delta_time());

            self.main.input.begin_frame();

            for event in self.main.event_pump().poll_iter() {
                use sdl2::event::Event::*;
                use sdl2::event::WindowEvent;
                match event {
                    Quit { .. } => window_closed = true,

                    Window { win_event, .. } => {
                        if let WindowEvent::Resized(x, y) = win_event {
                            resized = Some((x as u32, y as u32));
                        }
                    }

                    KeyDown { keycode, repeat, .. } => {
                        if !repeat {
                            self.main.input.handle_keyboard_input(ElementState::Pressed, keycode);
                        }
                    }
                    KeyUp { keycode, .. } =>
                        self.main.input.handle_keyboard_input(ElementState::Released, keycode),

                    MouseButtonDown { mouse_btn, .. } =>
                        self.main.input.handle_mouse_input(ElementState::Pressed, mouse_btn),
                    MouseButtonUp { mouse_btn, .. } =>
                        self.main.input.handle_mouse_input(ElementState::Released, mouse_btn),
                    MouseMotion { x, y, .. } =>
                        self.main.input.handle_mouse_motion(x, y),

                    ControllerDeviceAdded { which, .. } =>
                        self.main.input.handle_controller_added(which),
                    ControllerDeviceRemoved { which, .. } =>
                        self.main.input.handle_controller_removed(which),
                    ControllerDeviceRemapped { which, .. } =>
                        self.main.input.handle_controller_remapped(which),
                    ControllerAxisMotion { which, axis, value, .. } =>
                        self.main.input.handle_controller_axis(which, axis, value),
                    ControllerButtonDown { which, button, .. } =>
                        self.main.input.handle_controller_button(which, ElementState::Pressed, button),
                    ControllerButtonUp { which, button, .. } =>
                        self.main.input.handle_controller_button(which, ElementState::Released, button),

                    _ => {}
                }
            }

            let cur_win_size = self.main.graphics.screen_size();
            if cur_win_size != win_size {
                resized = Some(cur_win_size);
                win_size = cur_win_size;
            }
            if let Some(size) = resized {
                self.app.resize(size, &self.main);
                resized = None;
            }

            self.app.step(&mut self.main);

            let time_elapsed = start_time.elapsed();
            self.main.frame_times.add(Time::duration_as_f64(time_elapsed));
            if time_elapsed < self.frame_duration {
                thread::sleep(self.frame_duration - time_elapsed);
            }
        }

        self.app.destroy(&self.main);
    }
}

pub struct ApplicationGDX {
    sdl_context: sdl2::Sdl,
    time: Time,
    graphics: Graphics,
    input: Input,

    frame_times: MovingAverage<f64>,
    delta_times: MovingAverage<f64>,
    should_exit: bool,
}

impl ApplicationGDX {
    fn new(config: &ApplicationGDXConfig) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let graphics = Graphics::new(config, &sdl_context);
        let input = Input::new(&sdl_context);

        Self {
            sdl_context,
            time: Time::new(),
            graphics,
            input,

            frame_times: MovingAverage::new(200),
            delta_times: MovingAverage::new(200),
            should_exit: false,
        }
    }

    pub fn time(&self) -> &Time {
        &self.time
    }

    pub fn graphics(&self) -> &Graphics {
        &self.graphics
    }

    pub fn graphics_mut(&mut self) -> &mut Graphics {
        &mut self.graphics
    }

    pub fn input(&self) -> &Input {
        &self.input
    }

    pub fn frame_time(&self) -> f64 {
        self.frame_times.average()
    }

    pub fn fps(&self) -> f64 {
        1.0 / self.delta_times.average()
    }

    pub fn set_should_exit(&mut self) {
        self.should_exit = true
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    fn event_pump(&self) -> sdl2::EventPump {
        self.sdl_context.event_pump()
            .unwrap()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }
}
