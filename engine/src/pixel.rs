//! Ported from `pkg/pixel/pixel.go`.

/// Simulator-space position of a pixel (also baked for the desktop sim layout).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub size: f32,
}

/// An RGB pixel with channels in the `[0.0, 1.0]` range.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pixel {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub pos: Position,
}

impl Pixel {
    pub const fn new() -> Self {
        Pixel {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            pos: Position {
                x: 0.0,
                y: 0.0,
                size: 0.0,
            },
        }
    }

    /// Set the color without clamping (mirrors `SetColor`).
    pub fn set_color(&mut self, r: f32, g: f32, b: f32) {
        self.r = r;
        self.g = g;
        self.b = b;
    }

    /// Set the color, clamping each channel to `[0.0, 1.0]` (mirrors `SetClamped`).
    pub fn set_clamped(&mut self, r: f32, g: f32, b: f32) {
        self.r = clamp01(r);
        self.g = clamp01(g);
        self.b = clamp01(b);
    }

    pub fn set_position(&mut self, x: f32, y: f32, size: f32) {
        self.pos = Position { x, y, size };
    }

    /// Convert to an 8-bit `[r, g, b]` triple (mirrors the `output` stage's
    /// `Clamp255(channel * 255)`).
    pub fn to_rgb8(&self) -> [u8; 3] {
        [
            clamp255(self.r * 255.0),
            clamp255(self.g * 255.0),
            clamp255(self.b * 255.0),
        ]
    }
}

/// Clamp a float to `[0.0, 1.0]` (mirrors `pixel.Clamp01`).
pub fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

/// Clamp a float to a `u8` in `[0, 255]` (mirrors `pixel.Clamp255`).
pub fn clamp255(v: f32) -> u8 {
    if v < 0.0 {
        0
    } else if v > 255.0 {
        255
    } else {
        v as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps() {
        assert_eq!(clamp01(-0.5), 0.0);
        assert_eq!(clamp01(0.5), 0.5);
        assert_eq!(clamp01(1.5), 1.0);
        assert_eq!(clamp255(-1.0), 0);
        assert_eq!(clamp255(300.0), 255);
        assert_eq!(clamp255(127.0), 127);
    }

    #[test]
    fn set_clamped_clamps_channels() {
        let mut p = Pixel::new();
        p.set_clamped(2.0, -1.0, 0.5);
        assert_eq!((p.r, p.g, p.b), (1.0, 0.0, 0.5));
    }

    #[test]
    fn to_rgb8_full_white() {
        let mut p = Pixel::new();
        p.set_color(1.0, 1.0, 1.0);
        assert_eq!(p.to_rgb8(), [255, 255, 255]);
    }
}
