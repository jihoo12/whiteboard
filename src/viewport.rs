use crate::types::Point;

#[derive(Clone)]
pub struct Viewport {
    pub pan_x: f64,
    pub pan_y: f64,
    pub zoom:  f64,
}

impl Viewport {
    pub fn new() -> Self { Self { pan_x: 0.0, pan_y: 0.0, zoom: 1.0 } }

    pub fn to_world(&self, sx: f64, sy: f64) -> Point {
        Point { x: (sx - self.pan_x) / self.zoom,
                y: (sy - self.pan_y) / self.zoom }
    }

    /// World-space point → screen-space (x, y).
    pub fn to_screen(&self, wx: f64, wy: f64) -> (f64, f64) {
        (wx * self.zoom + self.pan_x, wy * self.zoom + self.pan_y)
    }

    pub fn apply(&self, cr: &gtk::cairo::Context) {
        cr.translate(self.pan_x, self.pan_y);
        cr.scale(self.zoom, self.zoom);
    }

    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.pan_x += dx;
        self.pan_y += dy;
    }

    pub fn zoom_around(&mut self, sx: f64, sy: f64, factor: f64) {
        let wx = (sx - self.pan_x) / self.zoom;
        let wy = (sy - self.pan_y) / self.zoom;
        self.zoom = (self.zoom * factor).clamp(0.05, 50.0);
        self.pan_x = sx - wx * self.zoom;
        self.pan_y = sy - wy * self.zoom;
    }
}