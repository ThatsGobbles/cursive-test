use std::sync::Arc;
use std::sync::Mutex;

use cursive::Vec2;
use cursive::views::Canvas;
use cursive::views::Dialog;
use cursive::traits::Scrollable;
use cursive::theme::Color;
use cursive::theme::ColorStyle;

const BLOCK_B_0_8: char = ' ';
const BLOCK_B_1_8: char = '▁';
const BLOCK_B_2_8: char = '▂';
const BLOCK_B_3_8: char = '▃';
const BLOCK_B_4_8: char = '▄';
const BLOCK_B_5_8: char = '▅';
const BLOCK_B_6_8: char = '▆';
const BLOCK_B_7_8: char = '▇';
const BLOCK_B_8_8: char = '█';

const BLOCK_L_0_8: char = BLOCK_B_0_8;
const BLOCK_L_1_8: char = '▏';
const BLOCK_L_2_8: char = '▎';
const BLOCK_L_3_8: char = '▍';
const BLOCK_L_4_8: char = '▌';
const BLOCK_L_5_8: char = '▋';
const BLOCK_L_6_8: char = '▊';
const BLOCK_L_7_8: char = '▉';
const BLOCK_L_8_8: char = BLOCK_B_8_8;

const RESOLUTION: usize = 8;
const MAX_BAR_WIDTH: usize = 100;
const MAX_BAR_TICKS: usize = RESOLUTION * MAX_BAR_WIDTH;

fn get_l_char(offset: usize) -> char {
    match offset {
        0 => BLOCK_L_0_8,
        1 => BLOCK_L_1_8,
        2 => BLOCK_L_2_8,
        3 => BLOCK_L_3_8,
        4 => BLOCK_L_4_8,
        5 => BLOCK_L_5_8,
        6 => BLOCK_L_6_8,
        7 => BLOCK_L_7_8,
        8 => BLOCK_L_8_8,
        _ => panic!(),
    }
}

fn gen_l_r_bar(logical_len: usize) -> Vec<char> {
    let int = logical_len / RESOLUTION;
    let rem = logical_len % RESOLUTION;

    let num_chars = if rem == 0 { int } else { int + 1 };

    let mut chars = Vec::<char>::with_capacity(num_chars);

    for _ in 0..int { chars.push(BLOCK_L_8_8); }
    if rem != 0 { chars.push(get_l_char(rem)); }

    chars
}

#[derive(Copy, Clone)]
struct RGB(u8, u8, u8);

impl RGB {
    fn interpolate(&self, other: &RGB, f: f64) -> RGB {
        assert!(f >= 0.0 && f <= 1.0);
        let o_f = f;
        let i_f = 1.0 - f;
        RGB(
            (i_f * self.0 as f64 + o_f * other.0 as f64).round() as u8,
            (i_f * self.1 as f64 + o_f * other.1 as f64).round() as u8,
            (i_f * self.2 as f64 + o_f * other.2 as f64).round() as u8,
        )
    }
}

impl From<RGB> for Color {
    fn from(rgb: RGB) -> Self {
        Color::Rgb(rgb.0, rgb.1, rgb.2)
    }
}

impl From<RGB> for ColorStyle {
    fn from(rgb: RGB) -> Self {
        Color::from(rgb).into()
    }
}

struct ModelData {
    /// A callback sink is used to control the UI from the server
    /// (eg. force refresh, error popups)
    cb_sink: cursive::CbSink,
    num_ticks: usize,
}

type Model = Arc<Mutex<ModelData>>;

fn build_spectrum_view(model: Model, size: Vec2) -> impl cursive::view::View {
    const C_A: RGB = RGB(0xc6, 0x2a, 0x81);
    const C_B: RGB = RGB(0x67, 0xe8, 0xce);

    Canvas::new(model)
        .with_draw(|model, printer| {
            let model = model.lock().unwrap();

            let chars = gen_l_r_bar(model.num_ticks);
            let num_chars = chars.len();

            for (i, ch) in chars.into_iter().enumerate() {
                let factor = i as f64 / ((MAX_BAR_WIDTH - 1) as f64);
                let s = ch.to_string();
                printer.print((1, 1), &format!("{}, {}, {}", i, MAX_BAR_WIDTH - 1, num_chars));
                printer.with_color(C_A.interpolate(&C_B, factor).into(), |p| p.print((i, 10), &s));
            }
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
                if model.num_ticks > MAX_BAR_TICKS { break; }
            }
            std::thread::sleep_ms(5);
        }
    });
}

pub fn run() {
    let mut siv = cursive::Cursive::default();

    // Build a shared model
    let model = Arc::new(Mutex::new(ModelData {
        cb_sink: siv.cb_sink().clone(),
        num_ticks: 0,
    }));

    // // Start the TCP server in a thread
    // start_server(Arc::clone(&model));

    // Build the UI from the model
    siv.add_layer(
        Dialog::around(
            build_spectrum_view(Arc::clone(&model), Vec2::new(200, 20))
        )
        .button("Quit", |s| s.quit()),
    );

    begin_counting(Arc::clone(&model));

    siv.run();
}
