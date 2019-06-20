use std::sync::Arc;
use std::sync::Mutex;

use cursive::Vec2;
use cursive::views::Canvas;
use cursive::views::Dialog;
use cursive::traits::Scrollable;
use cursive::theme::Color;
use cursive::theme::ColorStyle;

#[derive(Copy, Clone)]
struct RGB(u8, u8, u8);

impl RGB {
    fn interpolate(&self, other: &RGB, factor: f64) -> RGB {
        assert!(factor >= 0.0 && factor <= 1.0);
        let o_f = factor;
        let i_f = 1.0 - factor;
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
}

type Model = Arc<Mutex<ModelData>>;

fn build_spectrum_view(model: Model, size: Vec2) -> impl cursive::view::View {
    const N: usize = 100;
    const C_A: RGB = RGB(0xc6, 0x2a, 0x81);
    const C_B: RGB = RGB(0x67, 0xe8, 0xce);

    Canvas::new(model)
        .with_draw(|_model, printer| {
            printer.print_box((0,0), (6,4), true);

            for n in 0..N {
                let factor = n as f64 / ((N - 1) as f64);
                printer.with_color(C_A.interpolate(&C_B, factor).into(), |p| p.print((n, 10), "█"));
            }
        })
        // The required size will be set by the window layout, not by the printer!
        .with_required_size(move |_model, _req_size| size)
        .scrollable()
}

pub fn run() {
    let mut siv = cursive::Cursive::default();

    // Build a shared model
    let model = Arc::new(Mutex::new(ModelData {
        cb_sink: siv.cb_sink().clone(),
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

    siv.run();
}
