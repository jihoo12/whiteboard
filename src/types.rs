use gtk::cairo::Context;

/// A single point in 2D space.
#[derive(Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// A completed stroke made up of many sampled points.
#[derive(Clone)]
pub struct Stroke {
    pub points: Vec<Point>,
}

impl Stroke {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            points: vec![Point { x, y }],
        }
    }

    pub fn push(&mut self, x: f64, y: f64) {
        self.points.push(Point { x, y });
    }
}

/// Draw a smooth curve through `points` using Catmull-Rom → cubic Bézier conversion.
/// Falls back to line segments for very short strokes.
pub fn draw_smooth_stroke(cr: &Context, points: &[Point]) {
    if points.len() < 2 {
        return;
    }
    if points.len() == 2 {
        cr.move_to(points[0].x, points[0].y);
        cr.line_to(points[1].x, points[1].y);
        let _ = cr.stroke();
        return;
    }

    cr.move_to(points[0].x, points[0].y);

    // Convert Catmull-Rom spline to cubic Bézier segments.
    // For each interior segment [i, i+1] we need the "phantom" neighbours
    // p_{i-1} and p_{i+2}, clamping at the ends.
    let n = points.len();
    for i in 0..n - 1 {
        let p0 = if i == 0 { points[0] } else { points[i - 1] };
        let p1 = points[i];
        let p2 = points[i + 1];
        let p3 = if i + 2 < n { points[i + 2] } else { points[n - 1] };

        // Catmull-Rom tension = 0.5
        let cp1x = p1.x + (p2.x - p0.x) / 6.0;
        let cp1y = p1.y + (p2.y - p0.y) / 6.0;
        let cp2x = p2.x - (p3.x - p1.x) / 6.0;
        let cp2y = p2.y - (p3.y - p1.y) / 6.0;

        cr.curve_to(cp1x, cp1y, cp2x, cp2y, p2.x, p2.y);
    }

    let _ = cr.stroke();
}