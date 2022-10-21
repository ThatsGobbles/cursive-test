pub mod block_char;
pub mod block_line;
pub mod direction;
pub mod easing;
pub mod gradient_range;
pub mod remainder;

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use cursive::theme::ColorStyle;
use cursive::traits::Scrollable;
use cursive::views::Canvas;
use cursive::views::Dialog;
use cursive::CursiveExt;
use cursive::Printer;
use cursive::Vec2;

use crate::block_line::BlockLine;
use crate::direction::Direction;
use crate::easing::Easing;
use crate::gradient_range::GradientRange;

const MAX_BAR_LENGTH: usize = 60;

const GRAD_COLOR_1: (u8, u8, u8) = (0x42, 0x27, 0x5a);
const GRAD_COLOR_2: (u8, u8, u8) = (0x73, 0x4b, 0x6d);

const DEFAULT_EASING: Easing = Easing::BounceOut;

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
    let block_line = BlockLine::from_len_and_8ths(max_len, num_8ths, dir);

    for (i, (bc, cs)) in block_line.into_iter().zip(gradient_range).enumerate() {
        let cs = {
            if bc.needs_inversion() {
                invert_color_style(*cs)
            } else {
                *cs
            }
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

            print_gradient_line(
                &printer,
                0,
                0,
                &gradient_range,
                MAX_BAR_LENGTH,
                eased_8ths,
                Direction::Right,
            );
            print_gradient_line(
                &printer,
                0,
                1,
                &gradient_range,
                MAX_BAR_LENGTH,
                eased_8ths,
                Direction::Left,
            );
            print_gradient_line(
                &printer,
                0,
                0,
                &gradient_range,
                MAX_BAR_LENGTH,
                eased_8ths,
                Direction::Up,
            );
            print_gradient_line(
                &printer,
                1,
                0,
                &gradient_range,
                MAX_BAR_LENGTH,
                eased_8ths,
                Direction::Down,
            );
        })
        // The required size will be set by the window layout, not by the printer!
        .with_required_size(move |_model, _req_size| Vec2::new(MAX_BAR_LENGTH, MAX_BAR_LENGTH))
        .scrollable()
}

fn begin_counting(model: Model) {
    std::thread::spawn(move || loop {
        {
            let mut model = model.lock().unwrap();
            if model.lin_8ths > MAX_BAR_LENGTH * 8 {
                break;
            }
            model
                .cb_sink
                .send(Box::new(cursive::Cursive::noop))
                .unwrap();
            model.lin_8ths += 1;
        }
        std::thread::sleep(Duration::from_millis(5));
    });
}

pub fn main() {
    let mut siv = cursive::Cursive::default();

    siv.add_global_callback('q', |s| s.quit());

    // Build a shared model
    let model = Arc::new(Mutex::new(ModelData {
        cb_sink: siv.cb_sink().clone(),
        lin_8ths: 0,
    }));

    // Build the UI from the model
    siv.add_layer(Dialog::around(build_spectrum_view(Arc::clone(&model))));

    begin_counting(Arc::clone(&model));

    siv.run();
}
