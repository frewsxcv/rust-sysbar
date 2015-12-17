#![allow(non_snake_case)]

use std::collections::HashMap;
use std::sync::{Arc,RwLock};
use std::mem;

extern crate objc_foundation;
extern crate libc;

extern crate objc_id;
use objc_id::Id;

extern crate core_graphics;
use core_graphics::base::CGFloat;

#[macro_use]
extern crate objc;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};

extern crate cocoa;
use cocoa::base::{selector, nil, YES, id, class, BOOL};
use cocoa::foundation::{NSAutoreleasePool, NSString};
use cocoa::appkit::{NSApp,
					NSApplication,
					NSWindow,
					NSMenu, NSMenuItem, NSRunningApplication,
					NSApplicationActivateIgnoringOtherApps};

pub trait NSStatusBar {
	unsafe fn systemStatusBar(_: Self) -> id {
		msg_send![class("NSStatusBar"), systemStatusBar]
	}

	unsafe fn statusItemWithLength(self, len:CGFloat) -> id;
}

impl NSStatusBar for id {
	unsafe fn statusItemWithLength(self, len:CGFloat) -> id {
		msg_send![self, statusItemWithLength:len]
	}
}

pub trait NSStatusItem {
	unsafe fn setHighlightMode_(self, mode:BOOL);
	unsafe fn setMenu_(self, menu:id);
	unsafe fn statusBar(self) -> id;
}

impl NSStatusItem for id {
	unsafe fn setHighlightMode_(self, mode:BOOL){
		msg_send![self, setHighlightMode:mode]
	}
	unsafe fn statusBar(self) -> id {
		msg_send![self, statusBar]
	}
	unsafe fn setMenu_(self,menu:id) {
		msg_send![self, setMenu:menu]
	}
}

use objc::Message;
use objc_foundation::{INSObject, NSObject};

macro_rules! decl_objc_callback {
    ($name:ident, $cbs_name:ident) => (
		// this code is pretty much a rip off of
		// https://github.com/SSheldon/rust-objc-foundation/blob/master/examples/custom_class.rs

		// would be nice to use some mangled ident names here base on $name,
		// (and avoid the need for $cbs_name)
		// but concat_idents! doesn't work in the cases that I want.
		enum $name {};
		unsafe impl Message for $name { }

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
			fn from(cb:Box<Fn() -> ()>) -> Id<$name> {
				let cbs = $cbs_name {
					cb: cb
				};
				let bcbs = Box::new(cbs);

				let ptr = Box::into_raw(bcbs);
				let ptr = ptr as *mut libc::c_void as u64;
				println!("{}", ptr);
				let mut oid = $name::new();
				(*oid).setptr(ptr);
				oid
			}

			fn setptr(&mut self, uptr: u64) {
		        unsafe {
		            let obj =  &mut *(self as *mut _ as *mut Object);
					println!("setting the ptr: {}", uptr);
		            obj.set_ivar("_cbptr", uptr);
		        }
		    }
		}

		// TODO: Drop for $name doesn't get called, probably because objc manages the memory and
		// releases it for us.  so we leak the boxed callback right now.

		impl INSObject for $name {
			fn class() -> &'static Class {
				let cname = stringify!($name);

				let mut klass = Class::get(cname);
				if klass.is_none() {
					println!("registering class for {}", cname);
					let superclass = NSObject::class();
					let mut decl = ClassDecl::new(superclass, &cname).unwrap();
					decl.add_ivar::<u64>("_cbptr");

					extern fn $name(this: &Object, _cmd: Sel) {
						println!("callback, getting the pointer");
						unsafe {
							let pval:u64 = *this.get_ivar("_cbptr");
							let ptr = pval as *mut libc::c_void;
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
						decl.add_method(sel!($name), $name as extern fn(&Object, Sel));
					}

					decl.register();
					klass = Class::get(cname);
				}
				klass.unwrap()
			}
		}
	);
}

fn create_dock_app() {
	unsafe {
		let _pool = NSAutoreleasePool::new(nil);
		let app = NSApp();
		app.activateIgnoringOtherApps_(YES);

		let item = NSStatusBar::systemStatusBar(nil).statusItemWithLength(-1.0);
		item.setHighlightMode_(YES);
		let title = NSString::alloc(nil).init_str("Hacktest");
		item.setTitle_(title);

		let hm:HashMap<String,String> = HashMap::new();
		let hm = Arc::new(RwLock::new(hm));

		let phm = hm.clone();
		let pclosure = move || {
			let mut hm = phm.write().unwrap();
			let size = hm.len();
			let k = format!("Prefs{}", size);
			hm.insert(k, "Bar".to_owned());

			println!("prefs selected, new hm {:?}", *hm);
		};

		decl_objc_callback!(PreferencesCallback, PrefCBS);
		let pobj = PreferencesCallback::from(Box::new(pclosure));


		let fhm = hm.clone();
		decl_objc_callback!(FredCallback, FredCBS);
		let fobj = FredCallback::from(Box::new(move || {
			let mut hm = fhm.write().unwrap();
			let size = hm.len();
			let k = format!("Fred{}", size);
			hm.insert(k, "Bar".to_owned());

			println!("fred selected, new hm {:?}", *hm);
		}));

		// make menu
		let menu = NSMenu::new(nil).autorelease();
		let no_key = NSString::alloc(nil).init_str("");

		{
			{
				let itemtitle = NSString::alloc(nil).init_str("PREFS");
				let action = sel!(PreferencesCallback);
				let item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(itemtitle, action, no_key);
				let _: () = msg_send![item, setTarget:pobj];
				menu.addItem_(item);
			}

			{
				let itemtitle = NSString::alloc(nil).init_str("Fred");
				let action = sel!(FredCallback);
				let item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(itemtitle, action, no_key);
				let _: () = msg_send![item, setTarget:fobj];
				menu.addItem_(item);
			}
		}

		let pref_item = NSString::alloc(nil).init_str("Quit");
		let pref_action = selector("terminate:");
		let menuitem = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(pref_item, pref_action, no_key);

		menu.addItem_(menuitem);

		item.setMenu_(menu);

		println!("ok");

		// run!
		let current_app = NSRunningApplication::currentApplication(nil);
		current_app.activateWithOptions_(NSApplicationActivateIgnoringOtherApps);
		app.run();
	}
}

fn main() {
	create_dock_app();
}
