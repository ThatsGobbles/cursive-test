
use cursive::theme::Color;
use cursive::theme::ColorType;
use cursive::theme::ColorStyle;
use cursive::theme::PaletteColor;

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
