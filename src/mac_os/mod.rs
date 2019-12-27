#![allow(non_snake_case)]

use std::mem;

extern crate objc;

pub use objc::Message;

extern crate cocoa;
pub use self::cocoa::appkit::{
    NSApp, NSApplication, NSApplicationActivateIgnoringOtherApps, NSMenu, NSMenuItem,
    NSRunningApplication, NSStatusBar, NSStatusItem, NSWindow,
};
pub use self::cocoa::base::{nil, YES /* id, class, BOOL */};

extern crate libc;
pub use self::libc::c_void;
pub use self::objc::declare::ClassDecl;
pub use self::objc::runtime::{Class, Object, Sel};

extern crate objc_id;
pub use self::objc_id::Id;

extern crate objc_foundation;
pub use self::cocoa::foundation::{NSAutoreleasePool, NSString};
pub use self::objc_foundation::{INSObject, NSObject};

pub struct MacOsSysbar {
    name: String,
    menu: *mut objc::runtime::Object,
    pool: *mut objc::runtime::Object,
}

impl Drop for MacOsSysbar {
    fn drop(&mut self) {
        unsafe { self.pool.drain() }
    }
}

impl MacOsSysbar {
    pub fn new(name: &str) -> Self {
        unsafe {
            let pool = NSAutoreleasePool::new(nil);
            MacOsSysbar {
                name: name.to_owned(),
                pool,
                menu: NSMenu::new(nil).autorelease(),
            }
        }
    }
    #[allow(clippy::let_unit_value)]
    pub fn add_item(&mut self, label: &str, cbs: Box<dyn Fn() -> ()>) {
        unsafe {
            let cb_obj = Callback::from(cbs);

            let no_key = NSString::alloc(nil).init_str(""); // TODO want this eventually

            let itemtitle = NSString::alloc(nil).init_str(label);
            let action = sel!(call);
            let item = NSMenuItem::alloc(nil)
                .initWithTitle_action_keyEquivalent_(itemtitle, action, no_key);
            // Type inferance fails here, but we don't really
            // care about the return values so assigning
            // to _ with a () type annotation fixes a compile
            // time error
            let _: () = msg_send![item, setTarget: cb_obj];

            NSMenu::addItem_(self.menu, item);
        }
    }

    // TODO: allow user callback
    pub fn add_quit_item(&mut self, label: &str) {
        unsafe {
            let no_key = NSString::alloc(nil).init_str("");
            let pref_item = NSString::alloc(nil).init_str(label);
            let pref_action = sel!(terminate:);
            let menuitem = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
                pref_item,
                pref_action,
                no_key,
            );

            self.menu.addItem_(menuitem);
        }
    }

    pub fn display(&mut self) {
        unsafe {
            let app = NSApp();
            app.activateIgnoringOtherApps_(YES);

            let item = NSStatusBar::systemStatusBar(nil).statusItemWithLength_(-1.0);
            let title = NSString::alloc(nil).init_str(&self.name);
            item.setTitle_(title);
            item.setMenu_(self.menu);

            let current_app = NSRunningApplication::currentApplication(nil);
            current_app.activateWithOptions_(NSApplicationActivateIgnoringOtherApps);
            app.run();
        }
    }
}

// this code is pretty much a rip off of
// https://github.com/SSheldon/rust-objc-foundation/blob/master/examples/custom_class.rs

enum Callback {}
unsafe impl Message for Callback {}

// SO.. some explanation is in order here.  We want to allow closure callbacks that
// can modify their environment.  But we can't keep them on the $name object because
// that is really just a stateless proxy for the objc object.  So we store them
// as numeric pointers values in "ivar" fields on that object.  But, if we store a pointer to the
// closure object, we'll run into issues with thin/fat pointer conversions (because
// closure objects are trait objects and thus fat pointers).  So we wrap the closure in
// another boxed object ($cbs_name), which, since it doesn't use traits, is actually a
// regular "thin" pointer, and store THAT pointer in the ivar.  But...so...oy.
struct CallbackState {
    cb: Box<dyn Fn() -> ()>,
}

impl Callback {
    fn from(cb: Box<dyn Fn() -> ()>) -> Id<Self> {
        let cbs = CallbackState { cb };
        let bcbs = Box::new(cbs);

        let ptr = Box::into_raw(bcbs);
        let ptr = ptr as *mut c_void as usize;
        let mut oid = <Callback as INSObject>::new();
        (*oid).setptr(ptr);
        oid
    }

    fn setptr(&mut self, uptr: usize) {
        unsafe {
            let obj = &mut *(self as *mut _ as *mut ::objc::runtime::Object);
            obj.set_ivar("_cbptr", uptr);
        }
    }
}

// TODO: Drop for $name doesn't get called, probably because objc manages the memory and
// releases it for us.  so we leak the boxed callback right now.

impl INSObject for Callback {
    fn class() -> &'static Class {
        let cname = "Callback";

        let mut klass = Class::get(cname);
        if klass.is_none() {
            let superclass = NSObject::class();
            let mut decl = ClassDecl::new(&cname, superclass).unwrap();
            decl.add_ivar::<usize>("_cbptr");

            extern "C" fn sysbar_callback_call(this: &Object, _cmd: Sel) {
                unsafe {
                    let pval: usize = *this.get_ivar("_cbptr");
                    let ptr = pval as *mut c_void;
                    let ptr = ptr as *mut CallbackState;
                    let bcbs: Box<CallbackState> = Box::from_raw(ptr);
                    {
                        (*bcbs.cb)();
                    }
                    mem::forget(bcbs);
                }
            }

            unsafe {
                decl.add_method(
                    sel!(call),
                    sysbar_callback_call as extern "C" fn(&Object, Sel),
                );
            }

            decl.register();
            klass = Class::get(cname);
        }
        klass.unwrap()
    }
}
