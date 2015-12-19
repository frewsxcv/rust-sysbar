#![allow(non_snake_case)]

use std::mem;

#[macro_use]
extern crate objc;
pub use objc::Message;

extern crate cocoa;
pub use cocoa::base::{selector, nil, YES /* id, class, BOOL */};
pub use cocoa::appkit::{NSApp, NSApplication, NSWindow, NSMenu, NSMenuItem, NSRunningApplication,
                        NSApplicationActivateIgnoringOtherApps};

extern crate libc;
pub use libc::c_void;
pub use objc::declare::ClassDecl;
pub use objc::runtime::{Class, Object, Sel};

extern crate objc_id;
pub use objc_id::Id;

mod objc_ext;
use objc_ext::{NSStatusBar, NSStatusItem};

extern crate objc_foundation;
pub use cocoa::foundation::{NSAutoreleasePool, NSString};
pub use objc_foundation::{INSObject, NSObject};

pub struct Barfly {
    name: String,
    pub menu: *mut objc::runtime::Object,
    pool: *mut objc::runtime::Object,
}

impl Barfly {
    pub fn new(name: &str) -> Self {
        unsafe {
            Barfly {
                name: name.to_owned(),
                pool: NSAutoreleasePool::new(::cocoa::base::nil), /* TODO: not sure about the consequences of creating this here */
                menu: ::cocoa::appkit::NSMenu::new(::cocoa::base::nil).autorelease(),
            }
        }
    }

    // TODO: allow user callback
    pub fn add_quit_item(&mut self, label: &str) {
        unsafe {
            let no_key = ::cocoa::foundation::NSString::alloc(::cocoa::base::nil).init_str("");
            let pref_item = ::cocoa::foundation::NSString::alloc(::cocoa::base::nil)
                                .init_str(label);
            let pref_action = selector("terminate:");
            let menuitem = ::cocoa::appkit::NSMenuItem::alloc(::cocoa::base::nil)
                               .initWithTitle_action_keyEquivalent_(pref_item, pref_action, no_key);

            self.menu.addItem_(menuitem);
        }
    }

    pub fn display(self) {
        unsafe {
            let app = NSApp();
            app.activateIgnoringOtherApps_(YES);

            let item = NSStatusBar::systemStatusBar(::cocoa::base::nil).statusItemWithLength(-1.0);
            item.setHighlightMode_(YES);
            let title = ::cocoa::foundation::NSString::alloc(::cocoa::base::nil)
                            .init_str(&self.name);
            item.setTitle_(title);
            item.setMenu_(self.menu);

            let current_app = NSRunningApplication::currentApplication(::cocoa::base::nil);
            current_app.activateWithOptions_(NSApplicationActivateIgnoringOtherApps);
            app.run();
        }
    }
}

#[macro_export]
macro_rules! decl_objc_callback {
    ($name:ident, $cbs_name:ident) => (
        // this code is pretty much a rip off of
        // https://github.com/SSheldon/rust-objc-foundation/blob/master/examples/custom_class.rs

        // would be nice to use some mangled ident names here base on $name,
        // (and avoid the need for $cbs_name)
        // but concat_idents! doesn't work in the cases that I want.
        enum $name {};
        unsafe impl $crate::Message for $name { }

        // SO.. some explanation is in order here.  We want to allow closure callbacks that
        // can modify their environment.  But we can't keep them on the $name object because
        // that is really just a stateless proxy for the objc object.  So we store them
        // as numeric pointers values in "ivar" fields on that object.  But, if we store a pointer to the
        // closure object, we'll run into issues with thin/fat pointer conversions (because
        // closure objects are trait objects and thus fat pointers).  So we wrap the closure in
        // another boxed object ($cbs_name), which, since it doesn't use traits, is actually a
        // regular "thin" pointer, and store THAT pointer in the ivar.  But...so...oy.
        struct $cbs_name {
            cb: Box<Fn() -> ()>
        }

        impl $name {
            fn from(cb:Box<Fn() -> ()>) -> $crate::Id<$name> {
                let cbs = $cbs_name {
                    cb: cb
                };
                let bcbs = Box::new(cbs);

                let ptr = Box::into_raw(bcbs);
                let ptr = ptr as *mut $crate::c_void as u64;
                println!("{}", ptr);
                let mut oid = <$name as $crate::INSObject>::new();
                (*oid).setptr(ptr);
                oid
            }

            fn setptr(&mut self, uptr: u64) {
                unsafe {
                    let obj =  &mut *(self as *mut _ as *mut ::objc::runtime::Object);
                    println!("setting the ptr: {}", uptr);
                    obj.set_ivar("_cbptr", uptr);
                }
            }
        }

        // TODO: Drop for $name doesn't get called, probably because objc manages the memory and
        // releases it for us.  so we leak the boxed callback right now.

        impl $crate::INSObject for $name {
            fn class() -> &'static $crate::Class {
                let cname = stringify!($name);

                let mut klass = $crate::Class::get(cname);
                if klass.is_none() {
                    println!("registering class for {}", cname);
                    let superclass = $crate::NSObject::class();
                    let mut decl = $crate::ClassDecl::new(superclass, &cname).unwrap();
                    decl.add_ivar::<u64>("_cbptr");

                    extern fn $name(this: &$crate::Object, _cmd: $crate::Sel) {
                        println!("callback, getting the pointer");
                        unsafe {
                            let pval:u64 = *this.get_ivar("_cbptr");
                            let ptr = pval as *mut $crate::c_void;
                            let ptr = ptr as *mut $cbs_name;
                            let bcbs:Box<$cbs_name> = Box::from_raw(ptr);
                            {
                                println!("cb test from cb");
                                (*bcbs.cb)();
                            }
                            mem::forget(bcbs);
                        }
                    }

                    unsafe {
                        decl.add_method(sel!($name), $name as extern fn(&$crate::Object, $crate::Sel));
                    }

                    decl.register();
                    klass = $crate::Class::get(cname);
                }
                klass.unwrap()
            }
        }
    );
}

#[macro_export]
macro_rules! add_fly_item {
    ($fly:expr, $menuItem:expr, $name:ident, $cbs_name:ident, $cbs:expr) => (
        unsafe {
            decl_objc_callback!($name, $cbs_name);
            let cb_obj = $name::from($cbs);

            let astring = $crate::NSString::alloc($crate::nil);
            let no_key = $crate::NSString::init_str(astring,""); // TODO want this eventually

            let astring = $crate::NSString::alloc($crate::nil);
            let itemtitle = $crate::NSString::init_str(astring,$menuItem);
            let action = sel!($name);
            let aitem = $crate::NSMenuItem::alloc($crate::nil);
            let item = $crate::NSMenuItem::initWithTitle_action_keyEquivalent_(aitem, itemtitle, action, no_key);
            let _: () = msg_send![item, setTarget:cb_obj];

            $crate::NSMenu::addItem_($fly.menu, item);
        }
    )
}


#[test]
fn it_works() {
    let bf = Barfly::new("Test");
    add_fly_item!(&bf, "Test", TestCB, TestCBS, Box::new(|| {}));

}
