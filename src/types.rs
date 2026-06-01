use gtk::cairo::Context;

/// A single point in 2D **world** space.
#[derive(Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// A completed stroke made up of sampled world-space points.
#[derive(Clone)]
pub struct Stroke {
    pub points: Vec<Point>,
}

impl Stroke {
    pub fn new(x: f64, y: f64) -> Self {
        Self { points: vec![Point { x, y }] }
    }

    pub fn push(&mut self, x: f64, y: f64) {
        if let Some(last) = self.points.last() {
            let dx = x - last.x;
            let dy = y - last.y;
            if dx * dx + dy * dy < 4.0 { return; }
        }
        self.points.push(Point { x, y });
    }

    pub fn simplify(&mut self, epsilon: f64) {
        if self.points.len() >= 3 {
            self.points = rdp(&self.points, epsilon);
        }
    }
}

fn dist_sq_to_segment(p: Point, a: Point, b: Point) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let len_sq = dx * dx + dy * dy;
    if len_sq == 0.0 {
        return (p.x-a.x).powi(2) + (p.y-a.y).powi(2);
    }
    let t = ((p.x-a.x)*dx + (p.y-a.y)*dy / len_sq).clamp(0.0, 1.0);
    (a.x + t*dx - p.x).powi(2) + (a.y + t*dy - p.y).powi(2)
}

fn rdp(points: &[Point], epsilon: f64) -> Vec<Point> {
    if points.len() < 3 { return points.to_vec(); }
    let (first, last) = (points[0], *points.last().unwrap());
    let (max_idx, max_dsq) = points[1..points.len()-1].iter().enumerate()
        .map(|(i, &p)| (i+1, dist_sq_to_segment(p, first, last)))
        .fold((0, 0.0f64), |(ai,ad),(bi,bd)| if bd>ad {(bi,bd)} else {(ai,ad)});
    if max_dsq > epsilon*epsilon {
        let mut left = rdp(&points[..=max_idx], epsilon);
        let right = rdp(&points[max_idx..], epsilon);
        left.pop();
        left.extend_from_slice(&right);
        left
    } else {
        vec![first, last]
    }
}

pub fn draw_smooth_stroke(cr: &Context, points: &[Point]) {
    match points.len() {
        0 | 1 => {}
        2 => { cr.move_to(points[0].x, points[0].y);
               cr.line_to(points[1].x, points[1].y); let _ = cr.stroke(); }
        n => {
            cr.move_to(points[0].x, points[0].y);
            for i in 0..n-1 {
                let p0 = if i==0 { points[0] } else { points[i-1] };
                let p1 = points[i];
                let p2 = points[i+1];
                let p3 = if i+2<n { points[i+2] } else { points[n-1] };
                cr.curve_to(
                    p1.x+(p2.x-p0.x)/6.0, p1.y+(p2.y-p0.y)/6.0,
                    p2.x-(p3.x-p1.x)/6.0, p2.y-(p3.y-p1.y)/6.0,
                    p2.x, p2.y,
                );
            }
            let _ = cr.stroke();
        }
    }
}