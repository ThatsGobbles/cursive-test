use std::sync::Arc;
use std::sync::Mutex;

use cursive::Vec2;
use cursive::views::Canvas;
use cursive::views::Dialog;
use cursive::traits::Scrollable;

struct ModelData {
    /// A callback sink is used to control the UI from the server
    /// (eg. force refresh, error popups)
    cb_sink: cursive::CbSink,
}

type Model = Arc<Mutex<ModelData>>;

fn build_spectrum_view(model: Model, size: Vec2) -> impl cursive::view::View {
    Canvas::new(model)
        .with_draw(|_model, printer| {
            printer.print_box((0,0), (6,4), true);
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
            build_spectrum_view(Arc::clone(&model), Vec2::new(80, 20))
        )
        .button("Quit", |s| s.quit()),
    );

    siv.run();
}
