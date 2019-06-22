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

// const BLOCK_B_0_8: char = ' ';
// const BLOCK_B_1_8: char = '▁';
// const BLOCK_B_2_8: char = '▂';
// const BLOCK_B_3_8: char = '▃';
// const BLOCK_B_4_8: char = '▄';
// const BLOCK_B_5_8: char = '▅';
// const BLOCK_B_6_8: char = '▆';
// const BLOCK_B_7_8: char = '▇';
// const BLOCK_B_8_8: char = '█';

// const BLOCK_L_0_8: char = ' ';
const BLOCK_L_1_8: char = '▏';
const BLOCK_L_2_8: char = '▎';
const BLOCK_L_3_8: char = '▍';
const BLOCK_L_4_8: char = '▌';
const BLOCK_L_5_8: char = '▋';
const BLOCK_L_6_8: char = '▊';
const BLOCK_L_7_8: char = '▉';
const BLOCK_L_8_8: char = '█';

const MAX_BAR_LENGTH: usize = 200;
const MAX_BAR_LENGTH_EIGHTHS: usize = MAX_BAR_LENGTH * 8;
const MAX_BAR_TICKS: usize = 1000;

const GRAD_COLOR_1_R: u8 = 0xbd;
const GRAD_COLOR_1_G: u8 = 0xc3;
const GRAD_COLOR_1_B: u8 = 0xc7;
const GRAD_COLOR_2_R: u8 = 0x2c;
const GRAD_COLOR_2_G: u8 = 0x3e;
const GRAD_COLOR_2_B: u8 = 0x50;

const GRAD_COLOR_1: RGB = (0xbd, 0xc3, 0xc7);
const GRAD_COLOR_2: RGB = (0x2c, 0x3e, 0x50);

fn gen_l_r_bar(logical_len: usize) -> Vec<char> {
    let int = logical_len / 8;
    let rem = logical_len % 8;

    let num_chars = if rem == 0 { int } else { int + 1 };

    let mut chars = Vec::<char>::with_capacity(num_chars);

    for _ in 0..int { chars.push(BLOCK_L_8_8); }
    let tail_char = match rem {
        1 => BLOCK_L_1_8,
        2 => BLOCK_L_2_8,
        3 => BLOCK_L_3_8,
        4 => BLOCK_L_4_8,
        5 => BLOCK_L_5_8,
        6 => BLOCK_L_6_8,
        7 => BLOCK_L_7_8,
        _ => { return chars },
    };

    chars.push(tail_char);

    chars
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
    pub fn as_ticks(&self) -> usize {
        (self.0 * 8) + self.1.as_ticks()
    }

    pub fn from_ticks(t: usize) -> Self {
        Self(t / 8, Remainder::from_ticks(t))
    }

    pub fn from_num_chars(n: usize) -> Self {
        Self(n, Remainder::E0)
    }

    pub fn chars_needed(&self) -> usize {
        self.0 + if self.1.as_ticks() == 0 { 0 } else { 1 }
    }
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

#[derive(Copy, Clone)]
enum TipSmoothing { PartialBlocks, Fade }

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

fn interpolate_rgb(rgb_a: RGB, rgb_b: RGB, curr: usize, max: usize) -> RGB {
    let f = (curr as f64 / max as f64).clamp(0.0, 1.0);
    (
        ((1.0 - f) * rgb_a.0 as f64 + f * rgb_b.0 as f64).round() as u8,
        ((1.0 - f) * rgb_a.1 as f64 + f * rgb_b.1 as f64).round() as u8,
        ((1.0 - f) * rgb_a.2 as f64 + f * rgb_b.2 as f64).round() as u8,
    )
}

struct GradientColorIter {
    color_a: RGB,
    color_b: RGB,
    n: usize,
    curr_step: usize,
}

impl GradientColorIter {
    pub fn new(color_a: RGB, color_b: RGB, n: usize) -> Self {
        Self {
            color_a,
            color_b,
            n,
            curr_step: 0,
        }
    }
}

impl Iterator for GradientColorIter {
    type Item = RGB;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_step < self.n {
            let ret = Some(
                if self.color_a != self.color_b && self.n > 1 {
                    interpolate_rgb(self.color_a, self.color_b, self.curr_step, self.n - 1)
                } else { self.color_a }
            );
            self.curr_step += 1;
            ret
        } else { None }
    }
}

struct GradientBlockLine(Vec<(BlockChar, ColorStyle)>);

impl GradientBlockLine {
    pub fn new(ts: TickSpan, max_blocks: usize, direction: Direction, color_a: RGB, color_b: RGB) -> Self {
        let gradient_color_iter =
            GradientColorIter::new(color_a, color_b, max_blocks)
            .map(|rgb| {
                ColorStyle {
                    front: ColorType::Color(Color::Rgb(rgb.0, rgb.1, rgb.2)),
                    back: ColorType::Palette(PaletteColor::View),
                }
            })
        ;
        let block_line = BlockLine::new(ts, max_blocks, direction);

        Self(block_line.0.into_iter().zip(gradient_color_iter).collect())
    }
}

impl IntoIterator for GradientBlockLine {
    type Item = (BlockChar, ColorStyle);
    type IntoIter = ::std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// impl Iterator for GradientBlockIter {
//     type Item = GradientBlock;

//     fn next(&mut self) -> Option<Self::Item> {
//         let max_ticks = self.max_blocks * 8;

//         if self.curr_ticks < max_ticks {
//             let curr_blocks = (self.curr_ticks / 8) + if self.curr_ticks % 8 == 0 { 0 } else { 1 };

//             // See if we need to interpolate a color gradient.
//             let color_style =
//                 if self.fg_color_a == self.fg_color_b {
//                     ColorStyle{
//                         front: ColorType::Color(Color::Rgb(self.fg_color_a.0, self.fg_color_a.1, self.fg_color_a.2)),
//                         back: ColorType::Palette(PaletteColor::View),
//                     }
//                 }
//                 else {
//                     // Colors are always interpolated linearly, and by number of blocks, not by number of ticks.
//                     let grad_color = interpolate_rgb(self.fg_color_a, self.fg_color_b, curr_blocks, self.max_blocks);
//                     ColorStyle{
//                         front: ColorType::Color(Color::Rgb(grad_color.0, grad_color.1, grad_color.2)),
//                         back: ColorType::Palette(PaletteColor::View),
//                     }
//                 }
//             ;
//             self.curr_ticks += 1;
//             None
//         }
//         else { None }
//     }
// }

#[derive(Copy, Clone)]
enum Easing {
    Linear,
    BounceOut,
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
                    const A: f64 = 7.5625;

                    let (t_off, a_off) =
                        if t < 1.0 / 2.75 { (0.0, 0.0) }
                        else if t < 2.0 / 2.75 { (1.5 / 2.75, 0.75) }
                        else if t < 2.5 / 2.75 { (2.25 / 2.75, 0.9375) }
                        else { (2.625 / 2.75, 0.984375) }
                    ;

                    let tt = t - t_off;
                    let f = (A * tt * tt) + a_off;

                    (step as f64 * f).round() as usize
                },
            }
        }
    }
}

// fn ease_out_bounce(t: f64) -> f64 {
//     let t = t.clamp(0.0, 1.0);
//     const A: f64 = 7.5625;

//     let (t_off, a_off) =
//         if t < 1.0 / 2.75 { (0.0, 0.0) }
//         else if t < 2.0 / 2.75 { (1.5 / 2.75, 0.75) }
//         else if t < 2.5 / 2.75 { (2.25 / 2.75, 0.9375) }
//         else { (2.625 / 2.75, 0.984375) }
//     ;

//     let tt = t - t_off;
//     (A * tt * tt) + a_off
// }

fn osc_increase(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    (t + (12.0 * std::f64::consts::PI * t).sin() / 24.0).clamp(0.0, 1.0)
}

fn interpolate_u8s(a: u8, b: u8, f: f64) -> u8 {
    let f = f.clamp(0.0, 1.0);
    ((1.0 - f) * a as f64 + f * b as f64).round() as u8
}

struct ModelData {
    /// A callback sink is used to control the UI from the server
    /// (eg. force refresh, error popups)
    cb_sink: cursive::CbSink,
    num_ticks: usize,
}

type Model = Arc<Mutex<ModelData>>;

fn build_spectrum_view(model: Model, size: Vec2) -> impl cursive::view::View {
    Canvas::new(model)
        .with_draw(|model, printer| {
            let model = model.lock().unwrap();

            let line = GradientBlockLine::new(
                TickSpan::from_ticks(model.num_ticks),
                MAX_BAR_LENGTH,
                Direction::Right,
                GRAD_COLOR_1,
                GRAD_COLOR_2,
            );

            let linear_t = model.num_ticks as f64 / MAX_BAR_TICKS as f64;
            // let t = ease_out_bounce(linear_t);
            let t = osc_increase(linear_t);
            let num_eighths = (MAX_BAR_LENGTH_EIGHTHS as f64 * t).round() as usize;

            let chars = gen_l_r_bar(num_eighths);

            for (i, (bc, cs)) in line.into_iter().enumerate() {
                printer.with_color(
                    cs,
                    |p| p.print((i, 0), bc.into()),
                );
            }

            // for (i, ch) in chars.into_iter().enumerate() {
            //     let s = ch.to_string();
            //     let factor = i as f64 / ((MAX_BAR_LENGTH - 1) as f64);

            //     let c_r = interpolate_u8s(GRAD_COLOR_1_R, GRAD_COLOR_2_R, factor);
            //     let c_g = interpolate_u8s(GRAD_COLOR_1_G, GRAD_COLOR_2_G, factor);
            //     let c_b = interpolate_u8s(GRAD_COLOR_1_B, GRAD_COLOR_2_B, factor);

            //     printer.with_color(
            //         Color::Rgb(c_r, c_g, c_b).into(),
            //         |p| p.print((i, 0), &s),
            //     );
            // }
        })
        // The required size will be set by the window layout, not by the printer!
        .with_required_size(move |_model, _req_size| size)
        .scrollable()
}

fn begin_counting(model: Model) {
    std::thread::spawn(move || {
        loop {
            {
                let mut model = model.lock().unwrap();
                model.num_ticks += 1;
                model
                    .cb_sink
                    .send(Box::new(cursive::Cursive::noop))
                    .unwrap();
                // if model.num_ticks >= MAX_BAR_TICKS { break; }
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
        num_ticks: 0,
    }));

    // Build the UI from the model
    siv.add_layer(
        Dialog::around(
            build_spectrum_view(Arc::clone(&model), Vec2::new(200, 2))
        )
    );

    begin_counting(Arc::clone(&model));

    siv.run();
}
