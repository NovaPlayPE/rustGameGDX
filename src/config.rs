pub struct ApplicationGDXConfig {
    fps: u8,
    resizable: bool,
    screen_size: (u32, u32),
    title: String,
    vsync: bool,
}

impl ApplicationGDXConfig {
    pub fn new() -> Self {
        ApplicationGDXConfig {
            fps: 60,
            screen_size: (800, 600),
            resizable: false,
            title: "Rust GDX Launcher".into(),
            vsync: true,
        }
    }

    pub fn with_fps(mut self, fps: u8) -> Self {
        self.fps = fps;
        self
    }

    pub fn fps(&self) -> u8 {
        self.fps
    }

    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn resizable(&self) -> bool {
        self.resizable
    }

    pub fn with_screen_size(mut self, screen_size: (u32, u32)) -> Self {
        self.screen_size = screen_size;
        self
    }

    pub fn screen_size(&self) -> (u32, u32) {
        self.screen_size
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.into();
        self
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn with_vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }

    pub fn vsync(&self) -> bool {
        self.vsync
    }
}
