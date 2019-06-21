use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use cursive::Vec2;
use cursive::views::Canvas;
use cursive::views::Dialog;
use cursive::traits::Scrollable;
use cursive::theme::Color;

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

            let linear_t = model.num_ticks as f64 / MAX_BAR_TICKS as f64;
            // let t = ease_out_bounce(linear_t);
            let t = osc_increase(linear_t);
            let num_eighths = (MAX_BAR_LENGTH_EIGHTHS as f64 * t).round() as usize;

            let chars = gen_l_r_bar(num_eighths);

            for (i, ch) in chars.into_iter().enumerate() {
                let s = ch.to_string();
                let factor = i as f64 / ((MAX_BAR_LENGTH - 1) as f64);

                let c_r = interpolate_u8s(GRAD_COLOR_1_R, GRAD_COLOR_2_R, factor);
                let c_g = interpolate_u8s(GRAD_COLOR_1_G, GRAD_COLOR_2_G, factor);
                let c_b = interpolate_u8s(GRAD_COLOR_1_B, GRAD_COLOR_2_B, factor);

                printer.with_color(
                    Color::Rgb(c_r, c_g, c_b).into(),
                    |p| p.print((i, 0), &s),
                );
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
                if model.num_ticks >= MAX_BAR_TICKS { break; }
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
            build_spectrum_view(Arc::clone(&model), Vec2::new(200, 1))
        )
    );

    begin_counting(Arc::clone(&model));

    siv.run();
}
