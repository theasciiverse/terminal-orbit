use drawille::Canvas;

/// Thin wrapper around drawille::Canvas that provides exact-dimension trimming.
/// drawille's rows() returns (h/4 + 1) rows of (w/2 + 1) chars; we trim to exact.
pub struct BrailleCanvas {
    inner:  Canvas,
    w_dots: u32,
    h_dots: u32,
}

impl BrailleCanvas {
    pub fn new(w_dots: u32, h_dots: u32) -> Self {
        BrailleCanvas {
            inner: Canvas::new(w_dots, h_dots),
            w_dots,
            h_dots,
        }
    }

    /// Set a braille dot at (x, y) with bounds checking.
    pub fn set(&mut self, x: u32, y: u32) {
        if x < self.w_dots && y < self.h_dots {
            self.inner.set(x, y);
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Draw a line between two braille-dot coordinates.
    /// Silently ignores calls where either endpoint is out of canvas bounds.
    pub fn line(&mut self, x1: u32, y1: u32, x2: u32, y2: u32) {
        if x1 < self.w_dots && y1 < self.h_dots && x2 < self.w_dots && y2 < self.h_dots {
            self.inner.line(x1, y1, x2, y2);
        }
    }

    /// Returns exactly `h_dots/4` rows, each exactly `w_dots/2` braille chars.
    pub fn rows(&self) -> Vec<String> {
        let rows_needed = (self.h_dots / 4) as usize;
        let cols_needed = (self.w_dots / 2) as usize;

        self.inner
            .rows()
            .into_iter()
            .take(rows_needed)
            .map(|row| {
                let chars: Vec<char> = row.chars().collect();
                if chars.len() >= cols_needed {
                    chars[..cols_needed].iter().collect()
                } else {
                    let mut s: String = chars.iter().collect();
                    while s.chars().count() < cols_needed {
                        s.push(' ');
                    }
                    s
                }
            })
            .collect()
    }
}
