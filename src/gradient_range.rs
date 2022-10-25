use cursive::theme::Color;
use cursive::theme::ColorStyle;
use cursive::theme::ColorType;
use cursive::theme::PaletteColor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct GRange {
    rgb_0: RGB,
    rgb_1: RGB,
    n: usize,
}

impl GRange {
    pub fn new(a: RGB, b: RGB, num_steps: usize) -> Self {
        Self {
            rgb_0: a,
            rgb_1: b,
            n: num_steps,
        }
    }

    pub fn len(&self) -> usize {
        self.n
    }

    pub fn get(&self, i: usize) -> Option<ColorStyle> {
        if i >= self.n {
            return None;
        }

        let cs = if self.n == 1 {
            ColorStyle {
                front: ColorType::Color(Color::Rgb(self.rgb_0.r, self.rgb_0.g, self.rgb_0.b)),
                back: ColorType::Palette(PaletteColor::View),
            }
        } else {
            let max_den = self.n - 1;

            // Calculate the intermediate color using linear interpolation.
            let f = i as f64 / max_den as f64;
            let interp = RGB {
                r: ((1.0 - f) * self.rgb_0.r as f64 + f * self.rgb_1.r as f64).round() as u8,
                g: ((1.0 - f) * self.rgb_0.g as f64 + f * self.rgb_1.g as f64).round() as u8,
                b: ((1.0 - f) * self.rgb_0.b as f64 + f * self.rgb_1.b as f64).round() as u8,
            };

            ColorStyle {
                front: ColorType::Color(Color::Rgb(interp.r, interp.g, interp.b)),
                back: ColorType::Palette(PaletteColor::View),
            }
        };

        Some(cs)
    }
}

pub struct GRangeIter {
    g_range: GRange,
    state: Option<usize>,
}

impl GRangeIter {
    pub fn new(g_range: GRange) -> Self {
        Self {
            g_range,
            state: Some(0),
        }
    }

    pub fn reset(&mut self) {
        self.state = Some(0);
    }
}

impl Iterator for GRangeIter {
    type Item = ColorStyle;

    fn next(&mut self) -> Option<Self::Item> {
        self.state.and_then(|i| {
            let ret = self.g_range.get(i);

            if ret.is_some() {
                self.state = i.checked_add(1);
            } else {
                self.state = None;
            }

            ret
        })
    }
}

pub struct GradientRange(Vec<ColorStyle>);

impl GradientRange {
    pub fn new(color_a: (u8, u8, u8), color_b: (u8, u8, u8), n: usize) -> Self {
        if color_a == color_b {
            let cs_a = ColorStyle {
                front: ColorType::Color(Color::Rgb(color_a.0, color_a.1, color_a.2)),
                back: ColorType::Palette(PaletteColor::View),
            };
            GradientRange(vec![cs_a; n])
        } else if n == 0 {
            GradientRange(vec![])
        } else if n == 1 {
            let cs_a = ColorStyle {
                front: ColorType::Color(Color::Rgb(color_a.0, color_a.1, color_a.2)),
                back: ColorType::Palette(PaletteColor::View),
            };
            GradientRange(vec![cs_a])
        } else {
            let mut v: Vec<_> = Vec::with_capacity(n);
            let max_den = n - 1;
            for i in 0..=max_den {
                let f = i as f64 / max_den as f64;
                let new_color = (
                    ((1.0 - f) * color_a.0 as f64 + f * color_b.0 as f64).round() as u8,
                    ((1.0 - f) * color_a.1 as f64 + f * color_b.1 as f64).round() as u8,
                    ((1.0 - f) * color_a.2 as f64 + f * color_b.2 as f64).round() as u8,
                );

                let new_cs = ColorStyle {
                    front: ColorType::Color(Color::Rgb(new_color.0, new_color.1, new_color.2)),
                    back: ColorType::Palette(PaletteColor::View),
                };

                v.push(new_cs);
            }

            GradientRange(v)
        }
    }
}

impl<'a> IntoIterator for &'a GradientRange {
    type Item = &'a ColorStyle;
    type IntoIter = ::std::slice::Iter<'a, ColorStyle>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}
