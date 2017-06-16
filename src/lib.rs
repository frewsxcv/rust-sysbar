#[cfg(target_os = "macos")]
mod osx;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

pub trait Barfly {
    fn new(name: &str) -> Self;
    fn add_item(&mut self, menuItem: &str, cbs: Box<Fn() -> ()>);
    fn add_quit_item(&mut self, label: &str);
    fn display(&mut self);
}

#[cfg(target_os = "macos")]
pub type PlatformFly = osx::OsxBarfly;

pub fn new(name: &str) -> PlatformFly {
    PlatformFly::new(name)
}

#[test]
fn it_works() {
    let mut bf = new("Test"); //this is barfly::new()
    bf.add_item("Test", Box::new(|| {}));
}
