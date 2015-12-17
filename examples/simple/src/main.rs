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

#[macro_use]
extern crate barfly;
use barfly;

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
