//! Turtle graphics interpretation of L-system strings.
//!
//! The turtle interprets L-system symbols as movement and drawing commands:
//! - `F`, `G`: Move forward while drawing
//! - `f`: Move forward without drawing
//! - `+`: Turn left by the current angle
//! - `-`: Turn right by the current angle
//! - `[`: Push state onto stack
//! - `]`: Pop state from stack

/// A 2D line segment produced by turtle interpretation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    /// Start point (x, y).
    pub x1: f64,
    pub y1: f64,
    /// End point (x, y).
    pub x2: f64,
    pub y2: f64,
}

impl Segment {
    /// Create a new segment.
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self { x1, y1, x2, y2 }
    }

    /// Length of the segment.
    pub fn length(&self) -> f64 {
        let dx = self.x2 - self.x1;
        let dy = self.y2 - self.y1;
        (dx * dx + dy * dy).sqrt()
    }

    /// Midpoint of the segment.
    pub fn midpoint(&self) -> (f64, f64) {
        ((self.x1 + self.x2) / 2.0, (self.y1 + self.y2) / 2.0)
    }
}

/// A turtle for interpreting L-system strings as geometric paths.
///
/// # Examples
///
/// ```
/// use l_system_rs::turtle::Turtle;
///
/// let mut turtle = Turtle::new(90.0);
/// turtle.forward(10.0);
/// turtle.turn_left();
/// turtle.forward(10.0);
/// let segments = turtle.segments();
/// assert_eq!(segments.len(), 2);
/// ```
#[derive(Debug, Clone)]
pub struct Turtle {
    /// Current x position.
    pub x: f64,
    /// Current y position.
    pub y: f64,
    /// Current heading in degrees (0 = right/east, 90 = up/north).
    pub heading: f64,
    /// Step length for forward movement.
    pub step_length: f64,
    /// Turn angle in degrees.
    pub angle: f64,
    /// Stack of saved states for branching.
    pub stack: Vec<(f64, f64, f64)>,
    /// Segments drawn so far.
    pub segments: Vec<Segment>,
    /// Whether the pen is down (drawing).
    pen_down: bool,
}

impl Turtle {
    /// Create a new turtle at the origin heading up (90°) with the given turn angle.
    pub fn new(angle: f64) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            heading: 90.0,
            step_length: 1.0,
            angle,
            stack: Vec::new(),
            segments: Vec::new(),
            pen_down: true,
        }
    }

    /// Create a turtle with custom step length.
    pub fn with_step(angle: f64, step_length: f64) -> Self {
        Self {
            step_length,
            ..Self::new(angle)
        }
    }

    /// Set the step length.
    pub fn set_step_length(&mut self, length: f64) {
        self.step_length = length;
    }

    /// Move forward by `step_length`, optionally drawing.
    pub fn forward(&mut self, distance: f64) {
        let rad = self.heading.to_radians();
        let new_x = self.x + distance * rad.cos();
        let new_y = self.y + distance * rad.sin();
        if self.pen_down {
            self.segments.push(Segment::new(self.x, self.y, new_x, new_y));
        }
        self.x = new_x;
        self.y = new_y;
    }

    /// Move forward by step_length (drawing).
    pub fn step(&mut self) {
        self.forward(self.step_length);
    }

    /// Move forward by step_length (without drawing).
    pub fn move_without_draw(&mut self) {
        let was_pen = self.pen_down;
        self.pen_down = false;
        self.forward(self.step_length);
        self.pen_down = was_pen;
    }

    /// Turn left by the configured angle.
    pub fn turn_left(&mut self) {
        self.heading += self.angle;
    }

    /// Turn right by the configured angle.
    pub fn turn_right(&mut self) {
        self.heading -= self.angle;
    }

    /// Push the current state onto the stack.
    pub fn push(&mut self) {
        self.stack.push((self.x, self.y, self.heading));
    }

    /// Pop the state from the stack.
    pub fn pop(&mut self) {
        if let Some((x, y, h)) = self.stack.pop() {
            self.x = x;
            self.y = y;
            self.heading = h;
        }
    }

    /// Interpret an L-system string, producing line segments.
    ///
    /// Standard interpretation:
    /// - `F`, `G`: move forward drawing
    /// - `f`: move forward without drawing
    /// - `+`: turn left
    /// - `-`: turn right
    /// - `[`: push state
    /// - `]`: pop state
    /// - Other symbols: ignored
    pub fn interpret(&mut self, lstring: &str) {
        for ch in lstring.chars() {
            match ch {
                'F' | 'G' => self.step(),
                'f' => self.move_without_draw(),
                '+' => self.turn_left(),
                '-' => self.turn_right(),
                '[' => self.push(),
                ']' => self.pop(),
                _ => {} // Ignore other symbols
            }
        }
    }

    /// Get all drawn segments.
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    /// Clear all segments and reset position.
    pub fn reset(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
        self.heading = 90.0;
        self.stack.clear();
        self.segments.clear();
    }

    /// Compute the bounding box of all segments.
    ///
    /// Returns `(min_x, min_y, max_x, max_y)`. Returns `None` if no segments.
    pub fn bounding_box(&self) -> Option<(f64, f64, f64, f64)> {
        if self.segments.is_empty() {
            return None;
        }
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;
        for seg in &self.segments {
            for &(x, y) in &[(seg.x1, seg.y1), (seg.x2, seg.y2)] {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
        Some((min_x, min_y, max_x, max_y))
    }

    /// Count the total number of segments.
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turtle_new() {
        let t = Turtle::new(90.0);
        assert_eq!(t.x, 0.0);
        assert_eq!(t.y, 0.0);
        assert_eq!(t.heading, 90.0);
    }

    #[test]
    fn test_forward_creates_segment() {
        let mut t = Turtle::new(90.0);
        t.forward(1.0);
        assert_eq!(t.segments.len(), 1);
        let seg = &t.segments[0];
        assert!((seg.x1).abs() < 1e-10);
        assert!((seg.y1).abs() < 1e-10);
        assert!((seg.x2).abs() < 1e-10);
        assert!((seg.y2 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_turn_and_forward() {
        let mut t = Turtle::new(90.0);
        t.turn_right(); // Now heading 0 (east)
        t.forward(1.0);
        assert_eq!(t.segments.len(), 1);
        let seg = &t.segments[0];
        assert!((seg.x2 - 1.0).abs() < 1e-10);
        assert!((seg.y2).abs() < 1e-10);
    }

    #[test]
    fn test_square() {
        let mut t = Turtle::new(90.0);
        for _ in 0..4 {
            t.forward(1.0);
            t.turn_right();
        }
        assert_eq!(t.segments.len(), 4);
        // Should return near the origin
        assert!((t.x).abs() < 1e-10);
        assert!((t.y).abs() < 1e-10);
    }

    #[test]
    fn test_push_pop() {
        let mut t = Turtle::new(90.0);
        t.forward(1.0);
        t.push();
        t.turn_right();
        t.forward(1.0);
        assert_eq!(t.segments.len(), 2);
        t.pop();
        assert!((t.x - 0.0).abs() < 1e-10);
        assert!((t.y - 1.0).abs() < 1e-10);
        assert_eq!(t.heading, 90.0);
    }

    #[test]
    fn test_move_without_draw() {
        let mut t = Turtle::new(90.0);
        t.move_without_draw();
        assert_eq!(t.segments.len(), 0);
        assert!((t.y - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_interpret_koch() {
        let mut t = Turtle::new(60.0);
        t.interpret("F+F--F+F");
        assert_eq!(t.segments.len(), 4);
    }

    #[test]
    fn test_interpret_branching() {
        let mut t = Turtle::new(25.0);
        t.interpret("F[+F]F[-F]F");
        assert!(t.segments.len() > 3);
    }

    #[test]
    fn test_segment_length() {
        let seg = Segment::new(0.0, 0.0, 3.0, 4.0);
        assert!((seg.length() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_segment_midpoint() {
        let seg = Segment::new(0.0, 0.0, 2.0, 4.0);
        let (mx, my) = seg.midpoint();
        assert!((mx - 1.0).abs() < 1e-10);
        assert!((my - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_bounding_box() {
        let mut t = Turtle::new(90.0);
        t.interpret("F+F+F");
        let bb = t.bounding_box().unwrap();
        // Just verify bounding box is valid
        assert!(bb.2 >= bb.0); // max_x >= min_x
        assert!(bb.3 >= bb.1); // max_y >= min_y
    }

    #[test]
    fn test_bounding_box_empty() {
        let t = Turtle::new(90.0);
        assert!(t.bounding_box().is_none());
    }

    #[test]
    fn test_reset() {
        let mut t = Turtle::new(90.0);
        t.forward(5.0);
        t.push();
        t.reset();
        assert_eq!(t.segments.len(), 0);
        assert_eq!(t.x, 0.0);
        assert!(t.stack.is_empty());
    }

    #[test]
    fn test_interpret_ignores_unknown() {
        let mut t = Turtle::new(90.0);
        t.interpret("FxF");
        assert_eq!(t.segments.len(), 2); // 'x' is ignored
    }

    #[test]
    fn test_segment_count() {
        let mut t = Turtle::new(90.0);
        t.forward(1.0);
        t.forward(1.0);
        assert_eq!(t.segment_count(), 2);
    }

    #[test]
    fn test_set_step_length() {
        let mut t = Turtle::new(90.0);
        t.set_step_length(5.0);
        t.step();
        assert!((t.y - 5.0).abs() < 1e-10);
    }
}
