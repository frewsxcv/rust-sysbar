extern crate core_graphics;
use self::core_graphics::base::CGFloat;

extern crate cocoa;
use self::cocoa::base::{id, class, BOOL};

pub trait NSStatusItem {
    unsafe fn setMenu_(self, menu: id);
    unsafe fn statusBar(self) -> id;
}

impl NSStatusItem for id {
    unsafe fn statusBar(self) -> id {
        msg_send![self, statusBar]
    }
    unsafe fn setMenu_(self, menu: id) {
        msg_send![self, setMenu: menu]
    }
}
