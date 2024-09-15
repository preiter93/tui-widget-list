use std::time::{Duration, Instant};

use ratatui::{text::Line, widgets::Widget};

pub struct FPSCounter {
    last_frame: Instant,
    frame_count: u32,
    fps: f32,
    hide: bool,
}

impl Default for FPSCounter {
    fn default() -> Self {
        Self {
            last_frame: Instant::now(),
            frame_count: 0,
            fps: 0.0,
            hide: false,
        }
    }
}

impl FPSCounter {
    pub fn update(&mut self) {
        self.frame_count += 1;
        let now = Instant::now();
        let duration = now.duration_since(self.last_frame);

        // Update FPS every second
        if duration >= Duration::from_secs(1) {
            self.fps = self.frame_count as f32 / duration.as_secs_f32();
            self.frame_count = 0;
            self.last_frame = now;
        }
    }

    pub fn toggle(&mut self) {
        self.hide = !self.hide;
    }

    pub fn get_fps(&self) -> f32 {
        self.fps
    }
}
impl Widget for &FPSCounter {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        if self.hide {
            return;
        }
        let text = format!("FPS: {:.2}", self.get_fps());
        Line::from(text).render(area, buf);
    }
}
