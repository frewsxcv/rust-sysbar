extern crate core_graphics;
use self::core_graphics::base::CGFloat;

extern crate cocoa;
use self::cocoa::base::{id, class, BOOL};

pub trait NSStatusItem {
    unsafe fn setHighlightMode_(self, mode: BOOL);
    unsafe fn setMenu_(self, menu: id);
    unsafe fn statusBar(self) -> id;
}

impl NSStatusItem for id {
    unsafe fn setHighlightMode_(self, mode: BOOL) {
        msg_send![self, setHighlightMode: mode]
    }
    unsafe fn statusBar(self) -> id {
        msg_send![self, statusBar]
    }
    unsafe fn setMenu_(self, menu: id) {
        msg_send![self, setMenu: menu]
    }
}
