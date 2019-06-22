use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use cursive::Vec2;
use cursive::views::Canvas;
use cursive::views::Dialog;
use cursive::traits::Scrollable;
use cursive::theme::Color;
use cursive::theme::ColorType;
use cursive::theme::ColorStyle;
use cursive::theme::PaletteColor;

const MAX_BAR_LENGTH: usize = 200;

const GRAD_COLOR_1: RGB = (0x42, 0x27, 0x5a);
const GRAD_COLOR_2: RGB = (0x73, 0x4b, 0x6d);

const DEFAULT_EASING: Easing = Easing::BounceOut;

#[derive(Copy, Clone)]
pub enum Easing {
    Linear,
    BounceOut,
    Oscillate,
    QuadraticEaseIn,
    QuadraticEaseOut,
}

impl Easing {
    pub fn pos(&self, step: usize, max_steps: usize) -> usize {
        let step = step.min(max_steps);

        if max_steps == 0 { 0 }
        else {
            match self {
                &Easing::Linear => step,
                &Easing::BounceOut => {
                    let t = step as f64 / max_steps as f64;

                    let (t_off, a_off) =
                        if t < 1.0 / 2.75 { (0.0, 0.0) }
                        else if t < 2.0 / 2.75 { (1.5 / 2.75, 0.75) }
                        else if t < 2.5 / 2.75 { (2.25 / 2.75, 0.9375) }
                        else { (2.625 / 2.75, 0.984375) }
                    ;

                    let tt = t - t_off;
                    let f = (7.5625 * tt * tt) + a_off;

                    (max_steps as f64 * f).round() as usize
                },
                &Easing::Oscillate => {
                    let t = step as f64 / max_steps as f64;
                    let f = (t + (8.0 * std::f64::consts::PI * t).sin() / 16.0).clamp(0.0, 1.0);

                    (max_steps as f64 * f).round() as usize
                },
                &Easing::QuadraticEaseIn => {
                    let t = step as f64 / max_steps as f64;
                    (max_steps as f64 * t * t).round() as usize
                },
                &Easing::QuadraticEaseOut => {
                    let t = step as f64 / max_steps as f64;
                    let f = (1.0 - t).clamp(0.0, 1.0);
                    (max_steps as f64 * (1.0 - (f * f))).round() as usize
                },
            }
        }
    }
}

type RGB = (u8, u8, u8);

#[derive(Copy, Clone, PartialEq, PartialOrd, Ord, Eq)]
enum Remainder {
    E0, E1, E2, E3, E4, E5, E6, E7,
}

impl Remainder {
    pub fn as_ticks(&self) -> usize {
        match self {
            &Remainder::E0 => 0,
            &Remainder::E1 => 1,
            &Remainder::E2 => 2,
            &Remainder::E3 => 3,
            &Remainder::E4 => 4,
            &Remainder::E5 => 5,
            &Remainder::E6 => 6,
            &Remainder::E7 => 7,
        }
    }

    pub fn from_ticks(ticks: usize) -> Self {
        match ticks % 8 {
            0 => Remainder::E0,
            1 => Remainder::E1,
            2 => Remainder::E2,
            3 => Remainder::E3,
            4 => Remainder::E4,
            5 => Remainder::E5,
            6 => Remainder::E6,
            7 => Remainder::E7,
            _ => unreachable!(),
        }
    }
}

struct TickSpan(usize, Remainder);

impl TickSpan {
    // pub fn as_ticks(&self) -> usize {
    //     (self.0 * 8) + self.1.as_ticks()
    // }

    pub fn from_ticks(t: usize) -> Self {
        Self(t / 8, Remainder::from_ticks(t))
    }

    // pub fn from_num_chars(n: usize) -> Self {
    //     Self(n, Remainder::E0)
    // }

    // pub fn chars_needed(&self) -> usize {
    //     self.0 + if self.1.as_ticks() == 0 { 0 } else { 1 }
    // }
}

#[derive(Copy, Clone)]
enum BlockChar {
    NL, R1, R2, R3, R4, R5, R6, R7, U1, U2, U3, U4, U5, U6, U7, FF,
}

impl<'a> From<&'a BlockChar> for &'static str {
    fn from(bc: &'a BlockChar) -> Self {
        match bc {
            &BlockChar::NL => " ",
            &BlockChar::R1 => "▏",
            &BlockChar::R2 => "▎",
            &BlockChar::R3 => "▍",
            &BlockChar::R4 => "▌",
            &BlockChar::R5 => "▋",
            &BlockChar::R6 => "▊",
            &BlockChar::R7 => "▉",
            &BlockChar::U1 => "▁",
            &BlockChar::U2 => "▂",
            &BlockChar::U3 => "▃",
            &BlockChar::U4 => "▄",
            &BlockChar::U5 => "▅",
            &BlockChar::U6 => "▆",
            &BlockChar::U7 => "▇",
            &BlockChar::FF => "█",
        }
    }
}

impl From<BlockChar> for &'static str {
    fn from(bc: BlockChar) -> Self {
        (&bc).into()
    }
}

impl From<(Remainder, Direction)> for BlockChar {
    fn from(t: (Remainder, Direction)) -> Self {
        let (rem, dir) = t;

        match (rem, dir) {
            (Remainder::E0, Direction::Right) => BlockChar::NL,
            (Remainder::E1, Direction::Right) => BlockChar::R1,
            (Remainder::E2, Direction::Right) => BlockChar::R2,
            (Remainder::E3, Direction::Right) => BlockChar::R3,
            (Remainder::E4, Direction::Right) => BlockChar::R4,
            (Remainder::E5, Direction::Right) => BlockChar::R5,
            (Remainder::E6, Direction::Right) => BlockChar::R6,
            (Remainder::E7, Direction::Right) => BlockChar::R7,
            (Remainder::E0, Direction::Up) => BlockChar::NL,
            (Remainder::E1, Direction::Up) => BlockChar::U1,
            (Remainder::E2, Direction::Up) => BlockChar::U2,
            (Remainder::E3, Direction::Up) => BlockChar::U3,
            (Remainder::E4, Direction::Up) => BlockChar::U4,
            (Remainder::E5, Direction::Up) => BlockChar::U5,
            (Remainder::E6, Direction::Up) => BlockChar::U6,
            (Remainder::E7, Direction::Up) => BlockChar::U7,
        }
    }
}

#[derive(Copy, Clone)]
enum Direction { Right, Up, /* Left, Down, */ }

// #[derive(Copy, Clone)]
// enum TipSmoothing { PartialBlocks, Fade }

struct BlockLine(Vec<BlockChar>);

impl BlockLine {
    pub fn new(ts: TickSpan, max_blocks: usize, direction: Direction) -> Self {
        if ts.0 >= max_blocks {
            Self(vec![BlockChar::FF; max_blocks])
        } else if ts.0 == 0 && ts.1 == Remainder::E0 {
            Self(vec![BlockChar::NL; max_blocks])
        } else {
            let mut blocks = Vec::<BlockChar>::with_capacity(max_blocks);

            for _ in 0..ts.0 { blocks.push(BlockChar::FF); }
            blocks.push(BlockChar::from((ts.1, direction)));
            for _ in 0..(max_blocks - ts.0 - 1) { blocks.push(BlockChar::NL); }

            assert_eq!(max_blocks, blocks.len());

            Self(blocks)
        }
    }
}

impl IntoIterator for BlockLine {
    type Item = BlockChar;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

struct GradientRange(Vec<RGB>);

impl GradientRange {
    pub fn new(color_a: RGB, color_b: RGB, n: usize) -> Self {
        if color_a == color_b {
            GradientRange(vec![color_a; n])
        } else if n == 0 {
            GradientRange(vec![])
        } else if n == 1 {
            GradientRange(vec![color_a])
        } else {
            let mut v: Vec<RGB> = Vec::with_capacity(n);
            let max_den = n - 1;
            for i in 0..=max_den {
                let f = i as f64 / max_den as f64;
                let new_color = (
                    ((1.0 - f) * color_a.0 as f64 + f * color_b.0 as f64).round() as u8,
                    ((1.0 - f) * color_a.1 as f64 + f * color_b.1 as f64).round() as u8,
                    ((1.0 - f) * color_a.2 as f64 + f * color_b.2 as f64).round() as u8,
                );

                v.push(new_color);
            }

            GradientRange(v)
        }
    }
}

impl<'a> IntoIterator for &'a GradientRange {
    type Item = &'a RGB;
    type IntoIter = ::std::slice::Iter<'a, RGB>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

struct ModelData {
    /// A callback sink is used to control the UI from the server
    /// (eg. force refresh, error popups)
    cb_sink: cursive::CbSink,
    lin_ticks: usize,
}

type Model = Arc<Mutex<ModelData>>;

fn build_spectrum_view(model: Model) -> impl cursive::view::View {
    let gradient_range = GradientRange::new(GRAD_COLOR_1, GRAD_COLOR_2, MAX_BAR_LENGTH);
    Canvas::new(model)
        .with_draw(move |model, printer| {
            let model = model.lock().unwrap();

            let eased_ticks = DEFAULT_EASING.pos(model.lin_ticks, MAX_BAR_LENGTH * 8);

            let block_line = BlockLine::new(
                TickSpan::from_ticks(eased_ticks),
                MAX_BAR_LENGTH,
                Direction::Right,
            );

            let gradient_color_style_iter =
                (&gradient_range)
                .into_iter()
                .map(|rgb| {
                    ColorStyle {
                        front: ColorType::Color(Color::Rgb(rgb.0, rgb.1, rgb.2)),
                        back: ColorType::Palette(PaletteColor::View),
                    }
                })
            ;

            // let line = GradientBlockLine::new(
            //     TickSpan::from_ticks(eased_ticks),
            //     MAX_BAR_LENGTH,
            //     Direction::Right,
            //     GRAD_COLOR_1,
            //     GRAD_COLOR_2,
            // );

            for (i, (bc, cs)) in block_line.into_iter().zip(gradient_color_style_iter).enumerate() {
                printer.with_color(
                    cs,
                    |p| p.print((i, 0), bc.into()),
                );
            }
        })
        // The required size will be set by the window layout, not by the printer!
        .with_required_size(move |_model, _req_size| Vec2::new(MAX_BAR_LENGTH, 2))
        .scrollable()
}

fn begin_counting(model: Model) {
    std::thread::spawn(move || {
        loop {
            {
                let mut model = model.lock().unwrap();
                if model.lin_ticks > MAX_BAR_LENGTH * 8 { break; }
                model
                    .cb_sink
                    .send(Box::new(cursive::Cursive::noop))
                    .unwrap();
                model.lin_ticks += 1;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
    });
}

pub fn run() {
    let mut siv = cursive::Cursive::default();

    siv.add_global_callback('q', |s| s.quit());

    // Build a shared model
    let model = Arc::new(Mutex::new(ModelData {
        cb_sink: siv.cb_sink().clone(),
        lin_ticks: 0,
    }));

    // Build the UI from the model
    siv.add_layer(
        Dialog::around(
            build_spectrum_view(Arc::clone(&model))
        )
    );

    begin_counting(Arc::clone(&model));

    siv.run();
}
