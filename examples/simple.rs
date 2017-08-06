extern crate sysbar;

fn main() {
    let mut bar = sysbar::Sysbar::new("Foo");

    bar.add_item(
        "Say 'bar'",
        Box::new(move || {
            println!("bar");
        }),
    );

    bar.add_quit_item("Quit");

    bar.display();
}
