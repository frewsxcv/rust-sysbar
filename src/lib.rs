#[cfg(target_os = "macos")]
mod mac_os;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

pub struct Sysbar(PlatformFly);

impl Sysbar {
    pub fn new(name: &str) -> Self {
        Sysbar(PlatformFly::new(name))
    }

    pub fn add_item(&mut self, label: &str, cbs: Box<Fn() -> ()>) {
        self.0.add_item(label, cbs)
    }

    pub fn add_quit_item(&mut self, label: &str) {
        self.0.add_quit_item(label)
    }

    pub fn display(&mut self) {
        self.0.display()
    }
}

#[cfg(target_os = "macos")]
type PlatformFly = mac_os::MacOsBarfly;

#[test]
fn it_works() {
    let mut bf = new("Test"); //this is barfly::new()
    bf.add_item("Test", Box::new(|| {}));
}
