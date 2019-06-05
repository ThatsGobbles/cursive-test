use cursive::Cursive;
use cursive::views::SelectView;
use cursive::views::Dialog;
use cursive::views::LinearLayout;
use cursive::views::Button;
use cursive::views::DummyView;
use cursive::views::EditView;
use cursive::traits::Boxable;
use cursive::traits::Identifiable;
use cursive::traits::Scrollable;

const ID_NAME_EDITOR: &str = "ID_NAME_EDITOR";
const ID_NAME_SELECTOR: &str = "ID_NAME_SELECTOR";

fn add_name(s: &mut Cursive) {
    fn ok(s: &mut Cursive, name: &str) {
        s.call_on_id(ID_NAME_SELECTOR, |view: &mut SelectView<String>| {
            view.add_item_str(name);
        });
        s.pop_layer();
    }

    // Adds a new name to the profile list.
    let editor = EditView::new()
        .on_submit(ok)
        .with_id(ID_NAME_EDITOR)
        .fixed_width(10)
    ;

    let dialog = Dialog::around(editor)
        .title("Enter a new name")
        .button("Ok", |s| {
            let name = s.call_on_id(ID_NAME_EDITOR, |view: &mut EditView| {
                view.get_content()
            }).unwrap();

            ok(s, &name);
        })
        .button("Cancel", |s| { s.pop_layer(); })
    ;

    s.add_layer(dialog);
}

fn delete_name(s: &mut Cursive) {
    let mut select = s.find_id::<SelectView<String>>(ID_NAME_SELECTOR).unwrap();

    match select.selected_id() {
        None => s.add_layer(Dialog::info("No name to remove")),
        Some(focus) => { select.remove_item(focus); },
    };
}

fn on_submit(s: &mut Cursive, name: &String) {
    s.pop_layer();

    let dialog = Dialog::text(format!("Name: {}\nAwesome: yes", name))
        .title(format!("{}'s info", name))
        .button("Quit", Cursive::quit)
    ;

    s.add_layer(dialog);
}

fn profile_example() {
    let select = SelectView::<String>::new()
        .on_submit(on_submit)
        .with_id(ID_NAME_SELECTOR)
        .scrollable()
        .fixed_size((10, 5))
    ;

    let buttons = LinearLayout::vertical()
        .child(Button::new("Add new", add_name))
        .child(Button::new("Delete", delete_name))
        .child(DummyView)
        .child(Button::new("Quit", Cursive::quit))
    ;

    let combined = LinearLayout::horizontal()
        .child(select)
        .child(DummyView)
        .child(buttons)
    ;

    let main = Dialog::around(combined)
        .title("Select a profile")
    ;

    let mut siv = Cursive::default();
    siv.add_layer(main);
    siv.run();
}

fn main() {
    profile_example();
}
