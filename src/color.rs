
fn clamp(v: f32, min: f32, max: f32) -> f32 {
    v.max(min).min(max)
}

fn round_u8(v: f32) -> u8 {
    (v + 0.5).floor() as u8
}

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub red: f32,   // 0..1
    pub green: f32, // 0..1
    pub blue: f32,  // 0..1
    pub alpha: f32, // 0..1
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            red: clamp(r, 0.0, 1.0),
            green: clamp(g, 0.0, 1.0),
            blue: clamp(b, 0.0, 1.0),
            alpha: clamp(a, 0.0, 1.0),
        }
    }

    pub fn from_hex_str(s: &str) -> Result<Self, String> {
        let hex = s.trim_start_matches('#');

        let value = u32::from_str_radix(hex, 16)
            .map_err(|_| "Invalid hex color".to_string())?;

        match hex.len() {
            6 => {
                let r = ((value >> 16) & 0xff) as u8;
                let g = ((value >> 8) & 0xff) as u8;
                let b = (value & 0xff) as u8;
                Ok(Color::from_rgba(r, g, b, 1.0))
            }
            8 => {
                let r = ((value >> 24) & 0xff) as u8;
                let g = ((value >> 16) & 0xff) as u8;
                let b = ((value >> 8) & 0xff) as u8;
                let a = (value & 0xff) as f32 / 255.0;
                Ok(Color::from_rgba(r, g, b, a))
            }
            _ => Err("Hex string must be 6 or 8 characters".to_string()),
        }
    }

    pub fn from_rgba(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a,
        )
    }

    // pub fn from_hex(hex: u32) -> Self {
    //     let r = ((hex >> 24) & 0xff) as f32 / 255.0;
    //     let g = ((hex >> 16) & 0xff) as f32 / 255.0;
    //     let b = ((hex >> 8) & 0xff) as f32 / 255.0;
    //     let a = (hex & 0xff) as f32 / 255.0;
    //     Self::new(r, g, b, a)
    // }

    pub fn from_hsv(h: f32, s: f32, v: f32, a: f32) -> Self {
        let h = h.rem_euclid(360.0);
        let s = clamp(s, 0.0, 100.0) / 100.0;
        let v = clamp(v, 0.0, 100.0) / 100.0;

        let f = |n: f32| {
            let k = (n + h / 60.0).rem_euclid(6.0);
            v - v * s * (k.min(4.0 - k).min(1.0)).max(0.0)
        };

        Self::new(f(5.0), f(3.0), f(1.0), a)
    }

    // pub fn from_hsl(h: f32, s: f32, l: f32, a: f32) -> Self {
    //     let h = h.rem_euclid(360.0);
    //     let s = clamp(s, 0.0, 100.0) / 100.0;
    //     let l = clamp(l, 0.0, 100.0) / 100.0;

    //     let a2 = s * l.min(1.0 - l);

    //     let f = |n: f32| {
    //         let k = (n + h / 30.0).rem_euclid(12.0);
    //         l - a2 * (k - 3.0).min(9.0 - k).min(1.0).max(-1.0)
    //     };

    //     Self::new(f(0.0), f(8.0), f(4.0), a)
    // }

    // pub fn to_rgba(&self) -> (u8, u8, u8, f32) {
    //     (
    //         round_u8(self.red * 255.0),
    //         round_u8(self.green * 255.0),
    //         round_u8(self.blue * 255.0),
    //         self.alpha,
    //     )
    // }

    pub fn to_hex(&self, with_alpha: bool) -> u32 {
        let r = round_u8(self.red * 255.0) as u32;
        let g = round_u8(self.green * 255.0) as u32;
        let b = round_u8(self.blue * 255.0) as u32;
        let a = round_u8(self.alpha * 255.0) as u32;

        let rgb = (r << 16) | (g << 8) | b;
        if with_alpha {
            (rgb << 8) | a
        } else {
            rgb
        }
    }

    pub fn to_css(&self, with_alpha: bool) -> String {
        if with_alpha {
            format!("#{:08x}", self.to_hex(true))
        } else {
            format!("#{:06x}", self.to_hex(false))
        }
    }

    pub fn to_hsv(&self) -> (f32, f32, f32) {
        let max = self.red.max(self.green).max(self.blue);
        let min = self.red.min(self.green).min(self.blue);
        let delta = max - min;

        let mut h = if delta == 0.0 {
            0.0
        } else if max == self.red {
            60.0 * ((self.green - self.blue) / delta)
        } else if max == self.green {
            60.0 * ((self.blue - self.red) / delta + 2.0)
        } else {
            60.0 * ((self.red - self.green) / delta + 4.0)
        };

        if h < 0.0 {
            h += 360.0;
        }

        let s = if max == 0.0 { 0.0 } else { delta / max };
        (h, s * 100.0, max * 100.0)
    }

    // pub fn to_hsl(&self) -> (f32, f32, f32) {
    //     let max = self.red.max(self.green).max(self.blue);
    //     let min = self.red.min(self.green).min(self.blue);
    //     let l = (max + min) / 2.0;

    //     let delta = max - min;
    //     let s = if delta == 0.0 {
    //         0.0
    //     } else {
    //         delta / (1.0 - (2.0 * l - 1.0).abs())
    //     };

    //     let h = self.to_hsv().0;
    //     (h, s * 100.0, l * 100.0)
    // }

    pub fn blend(&self, other: &Color, f: f32) -> Color {
        Color::new(
            (other.red - self.red) * f + self.red,
            (other.green - self.green) * f + self.green,
            (other.blue - self.blue) * f + self.blue,
            self.alpha,
        )
    }

    pub fn shade(&self, f: f32) -> Color {
        let t = if f < 0.0 { 0.0 } else { 1.0 };
        let p = f.abs();

        Color::new(
            (t - self.red) * p + self.red,
            (t - self.green) * p + self.green,
            (t - self.blue) * p + self.blue,
            self.alpha,
        )
    }

    pub fn brighten(&self, v: f32) -> Color {
        let (h, s, val) = self.to_hsv();
        Color::from_hsv(h, s, clamp(val + v, 0.0, 100.0), self.alpha)
    }

    // pub fn lighten(&self, v: f32) -> Color {
    //     let (h, s, l) = self.to_hsl();
    //     Color::from_hsl(h, s, clamp(l + v, 0.0, 100.0), self.alpha)
    // }

    // pub fn saturate(&self, v: f32) -> Color {
    //     let (h, s, val) = self.to_hsv();
    //     Color::from_hsv(h, clamp(s + v, 0.0, 100.0), val, self.alpha)
    // }

    // pub fn rotate(&self, v: f32) -> Color {
    //     let (h, s, val) = self.to_hsv();
    //     Color::from_hsv((h + v).rem_euclid(360.0), s, val, self.alpha)
    // }

    // pub fn luminance(&self) -> f32 {
    //     let f = |c: f32| {
    //         if c > 0.04045 {
    //             ((c + 0.055) / 1.055).powf(2.4)
    //         } else {
    //             c / 12.92
    //         }
    //     };

    //     0.2126 * f(self.red) + 0.7152 * f(self.green) + 0.0722 * f(self.blue)
    // }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_css(false))
    }
}
