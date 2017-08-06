#![allow(non_snake_case)]

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::mem;

extern crate sysbar;
use sysbar::Sysbar;

fn main() {
    let mut bar = Sysbar::new("Foo");

    // make a hash map for the callbacks to mess with
    let hm: HashMap<String, String> = HashMap::new();
    let hm = Arc::new(RwLock::new(hm));

    let phm = hm.clone();
    bar.add_item(
        "Prefs",
        Box::new(move || {
            let mut hm = phm.write().unwrap();
            let size = hm.len();
            let k = format!("Prefs{}", size);
            hm.insert(k, "Bar".to_owned());

            println!("prefs selected, new hm {:?}", *hm);
        }),
    );

    let fhm = hm.clone();
    bar.add_item(
        "Summon Herb",
        Box::new(move || {
            let mut hm = fhm.write().unwrap();
            let size = hm.len();
            let k = format!("Herb{}", size);
            hm.insert(k, "Bar".to_owned());

            println!(
                "Herb thinks you are a jerk and refuses to appear.  By the way, \
                                the hash map is: {:?}",
                *hm
            );
        }),
    );

    bar.add_quit_item("Quit");

    bar.display();
}
