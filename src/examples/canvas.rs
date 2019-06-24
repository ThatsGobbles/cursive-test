use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::cmp::Ordering;

use cursive::Vec2;
use cursive::Printer;
use cursive::views::Canvas;
use cursive::views::Dialog;
use cursive::traits::Scrollable;
use cursive::theme::Color;
use cursive::theme::ColorType;
use cursive::theme::ColorStyle;
use cursive::theme::PaletteColor;

const MAX_BAR_LENGTH: usize = 60;

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
    pub fn as_8ths(&self) -> usize {
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

    pub fn from_8ths(n_8ths: usize) -> Self {
        match n_8ths % 8 {
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

#[derive(Copy, Clone)]
enum BlockChar {
    NL,
    R1, R2, R3, R4, R5, R6, R7,
    U1, U2, U3, U4, U5, U6, U7,
    L1, L2, L3, L4, L5, L6, L7,
    D1, D2, D3, D4, D5, D6, D7,
    FF,
}

impl BlockChar {
    fn needs_inversion(&self) -> bool {
        match self {
            &BlockChar::L1 => true,
            &BlockChar::L2 => true,
            &BlockChar::L3 => true,
            &BlockChar::L4 => true,
            &BlockChar::L5 => true,
            &BlockChar::L6 => true,
            &BlockChar::L7 => true,
            &BlockChar::D1 => true,
            &BlockChar::D2 => true,
            &BlockChar::D3 => true,
            &BlockChar::D4 => true,
            &BlockChar::D5 => true,
            &BlockChar::D6 => true,
            &BlockChar::D7 => true,
            _ => false,
        }
    }
}

impl<'a> From<&'a BlockChar> for &'static str {
    fn from(bc: &'a BlockChar) -> Self {
        match bc {
            &BlockChar::NL => " ",
            &BlockChar::R1 | &BlockChar::L7 => "▏",
            &BlockChar::R2 | &BlockChar::L6 => "▎",
            &BlockChar::R3 | &BlockChar::L5 => "▍",
            &BlockChar::R4 | &BlockChar::L4 => "▌",
            &BlockChar::R5 | &BlockChar::L3 => "▋",
            &BlockChar::R6 | &BlockChar::L2 => "▊",
            &BlockChar::R7 | &BlockChar::L1 => "▉",
            &BlockChar::U1 | &BlockChar::D7 => "▁",
            &BlockChar::U2 | &BlockChar::D6 => "▂",
            &BlockChar::U3 | &BlockChar::D5 => "▃",
            &BlockChar::U4 | &BlockChar::D4 => "▄",
            &BlockChar::U5 | &BlockChar::D3 => "▅",
            &BlockChar::U6 | &BlockChar::D2 => "▆",
            &BlockChar::U7 | &BlockChar::D1 => "▇",
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
            (Remainder::E0, _) => BlockChar::NL,
            (Remainder::E1, Direction::Right) => BlockChar::R1,
            (Remainder::E2, Direction::Right) => BlockChar::R2,
            (Remainder::E3, Direction::Right) => BlockChar::R3,
            (Remainder::E4, Direction::Right) => BlockChar::R4,
            (Remainder::E5, Direction::Right) => BlockChar::R5,
            (Remainder::E6, Direction::Right) => BlockChar::R6,
            (Remainder::E7, Direction::Right) => BlockChar::R7,
            (Remainder::E1, Direction::Up) => BlockChar::U1,
            (Remainder::E2, Direction::Up) => BlockChar::U2,
            (Remainder::E3, Direction::Up) => BlockChar::U3,
            (Remainder::E4, Direction::Up) => BlockChar::U4,
            (Remainder::E5, Direction::Up) => BlockChar::U5,
            (Remainder::E6, Direction::Up) => BlockChar::U6,
            (Remainder::E7, Direction::Up) => BlockChar::U7,
            (Remainder::E1, Direction::Left) => BlockChar::L1,
            (Remainder::E2, Direction::Left) => BlockChar::L2,
            (Remainder::E3, Direction::Left) => BlockChar::L3,
            (Remainder::E4, Direction::Left) => BlockChar::L4,
            (Remainder::E5, Direction::Left) => BlockChar::L5,
            (Remainder::E6, Direction::Left) => BlockChar::L6,
            (Remainder::E7, Direction::Left) => BlockChar::L7,
            (Remainder::E1, Direction::Down) => BlockChar::D1,
            (Remainder::E2, Direction::Down) => BlockChar::D2,
            (Remainder::E3, Direction::Down) => BlockChar::D3,
            (Remainder::E4, Direction::Down) => BlockChar::D4,
            (Remainder::E5, Direction::Down) => BlockChar::D5,
            (Remainder::E6, Direction::Down) => BlockChar::D6,
            (Remainder::E7, Direction::Down) => BlockChar::D7,
        }
    }
}

#[derive(Copy, Clone)]
enum Direction { Right, Up, Left, Down, }

// #[derive(Copy, Clone)]
// enum TipSmoothing { PartialBlocks, Fade }

#[derive(Copy, Clone)]
struct BlockLine {
    /// Number of filled blocks.
    pub filled_blocks: usize,

    // Optional partial block and number of empty blocks after.
    // If `None`, then the block line is exactly full.
    pub tail: Option<(Remainder, usize)>,

    // Direction to produce blocks in.
    pub dir: Direction,
}

impl BlockLine {
    pub fn char_len(&self) -> usize {
        self.filled_blocks + if let Some((_, empty_blocks)) = self.tail { 1 + empty_blocks } else { 0 }
    }

    pub fn _rem(&self) -> Remainder {
        if let Some((rem, _)) = self.tail { rem }
        // A full block line has a remainder of 0.
        else { Remainder::E0 }
    }

    pub fn char_at(&self, index: usize) -> BlockChar {
        match (index.cmp(&self.filled_blocks), self.tail) {
            (Ordering::Less, _) => BlockChar::FF,
            (Ordering::Equal, Some((rem, _))) => BlockChar::from((rem, self.dir)),
            (_, _) => BlockChar::NL,
        }
    }

    pub fn from_len_and_8ths(max_len: usize, filled_8ths: usize, dir: Direction) -> Self {
        let filled_blocks = filled_8ths / 8;
        let res = if filled_blocks >= max_len {
            BlockLine {
                filled_blocks: max_len,
                tail: None,
                dir,
            }
        } else {
            BlockLine {
                filled_blocks,
                tail: Some((Remainder::from_8ths(filled_8ths), max_len - filled_blocks - 1)),
                dir,
            }
        };

        assert_eq!(max_len, res.char_len());
        res
    }
}

impl IntoIterator for BlockLine {
    type Item = BlockChar;
    type IntoIter = BlockLineIter;

    fn into_iter(self) -> Self::IntoIter {
        BlockLineIter {
            block_line: self,
            curr_ch_idx: 0,
        }
    }
}

#[derive(Copy, Clone)]
struct BlockLineIter {
    block_line: BlockLine,
    curr_ch_idx: usize
}

impl Iterator for BlockLineIter {
    type Item = BlockChar;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_ch_idx < self.block_line.char_len() {
            let bc = self.block_line.char_at(self.curr_ch_idx);
            self.curr_ch_idx += 1;
            Some(bc)
        } else { None }
    }
}

struct GradientRange(Vec<ColorStyle>);

impl GradientRange {
    pub fn new(color_a: RGB, color_b: RGB, n: usize) -> Self {
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

struct ModelData {
    /// A callback sink is used to control the UI from the server
    /// (eg. force refresh, error popups)
    cb_sink: cursive::CbSink,
    lin_8ths: usize,
}

type Model = Arc<Mutex<ModelData>>;

fn invert_color_style(cs: ColorStyle) -> ColorStyle {
    ColorStyle {
        front: cs.back,
        back: cs.front,
    }
}

fn print_gradient_line<'a>(
    printer: &Printer,
    x: usize,
    y: usize,
    gradient_range: &'a GradientRange,
    max_len: usize,
    num_8ths: usize,
    dir: Direction,
) {
    let block_line = BlockLine::from_len_and_8ths(
        max_len,
        num_8ths,
        dir,
    );

    for (i, (bc, cs)) in block_line.into_iter().zip(gradient_range).enumerate() {
        let cs = {
            if bc.needs_inversion() { invert_color_style(*cs) }
            else { *cs }
        };

        let (tx, ty) = match dir {
            Direction::Right => (x + i, y),
            Direction::Left => (x + (max_len - i) - 1, y),
            Direction::Down => (x, y + i),
            Direction::Up => (x, y + (max_len - i) - 1),
        };

        printer.with_color(cs, |p| p.print((tx, ty), bc.into()));
    }
}

fn build_spectrum_view(model: Model) -> impl cursive::view::View {
    let gradient_range = GradientRange::new(GRAD_COLOR_1, GRAD_COLOR_2, MAX_BAR_LENGTH);
    Canvas::new(model)
        .with_draw(move |model, printer| {
            let model = model.lock().unwrap();

            let eased_8ths = DEFAULT_EASING.pos(model.lin_8ths, MAX_BAR_LENGTH * 8);

            // print_gradient_line(&printer, 0, 0, &gradient_range, MAX_BAR_LENGTH, eased_8ths, Direction::Right);
            // print_gradient_line(&printer, 0, 1, &gradient_range, MAX_BAR_LENGTH, eased_8ths, Direction::Left);
            print_gradient_line(&printer, 0, 0, &gradient_range, MAX_BAR_LENGTH, eased_8ths, Direction::Up);
            print_gradient_line(&printer, 1, 0, &gradient_range, MAX_BAR_LENGTH, eased_8ths, Direction::Down);
        })
        // The required size will be set by the window layout, not by the printer!
        .with_required_size(move |_model, _req_size| Vec2::new(MAX_BAR_LENGTH, MAX_BAR_LENGTH))
        .scrollable()
}

fn begin_counting(model: Model) {
    std::thread::spawn(move || {
        loop {
            {
                let mut model = model.lock().unwrap();
                if model.lin_8ths > MAX_BAR_LENGTH * 8 { break; }
                model
                    .cb_sink
                    .send(Box::new(cursive::Cursive::noop))
                    .unwrap();
                model.lin_8ths += 1;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    });
}

pub fn run() {
    let mut siv = cursive::Cursive::default();

    siv.add_global_callback('q', |s| s.quit());

    // Build a shared model
    let model = Arc::new(Mutex::new(ModelData {
        cb_sink: siv.cb_sink().clone(),
        lin_8ths: 0,
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
