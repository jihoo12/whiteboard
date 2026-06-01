use crate::types::Point;

/// Tracks pan offset and zoom level for the infinite canvas.
///
/// World → screen:  sx = wx * zoom + pan_x
/// Screen → world:  wx = (sx - pan_x) / zoom
#[derive(Clone)]
pub struct Viewport {
    pub pan_x: f64,
    pub pan_y: f64,
    pub zoom: f64,
}

impl Viewport {
    pub fn new() -> Self {
        Self { pan_x: 0.0, pan_y: 0.0, zoom: 1.0 }
    }

    /// Convert a screen-space point to world space.
    pub fn to_world(&self, sx: f64, sy: f64) -> Point {
        Point {
            x: (sx - self.pan_x) / self.zoom,
            y: (sy - self.pan_y) / self.zoom,
        }
    }

    /// Apply the viewport transform to a Cairo context so subsequent drawing
    /// commands can use world-space coordinates directly.
    pub fn apply(&self, cr: &gtk::cairo::Context) {
        cr.translate(self.pan_x, self.pan_y);
        cr.scale(self.zoom, self.zoom);
    }

    /// Pan by a screen-space delta.
    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.pan_x += dx;
        self.pan_y += dy;
    }

    /// Zoom in/out around a screen-space anchor point.
    pub fn zoom_around(&mut self, screen_x: f64, screen_y: f64, factor: f64) {
        // Keep the world point under (screen_x, screen_y) fixed.
        let wx = (screen_x - self.pan_x) / self.zoom;
        let wy = (screen_y - self.pan_y) / self.zoom;
        self.zoom = (self.zoom * factor).clamp(0.05, 50.0);
        self.pan_x = screen_x - wx * self.zoom;
        self.pan_y = screen_y - wy * self.zoom;
    }
}