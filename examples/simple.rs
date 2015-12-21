#![allow(non_snake_case)]

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::mem;

extern crate barfly;
use barfly::Barfly;

fn main() {
    let mut fly = barfly::new("Barfly");

    // make a hash map for the callbacks to mess with
    let hm: HashMap<String, String> = HashMap::new();
    let hm = Arc::new(RwLock::new(hm));

    let phm = hm.clone();
    fly.add_item("Prefs",
                  Box::new(move || {
                      let mut hm = phm.write().unwrap();
                      let size = hm.len();
                      let k = format!("Prefs{}", size);
                      hm.insert(k, "Bar".to_owned());

                      println!("prefs selected, new hm {:?}", *hm);
                  }));

    let fhm = hm.clone();
    fly.add_item("Summon Herb",
                  Box::new(move || {
                      let mut hm = fhm.write().unwrap();
                      let size = hm.len();
                      let k = format!("Herb{}", size);
                      hm.insert(k, "Bar".to_owned());

                      println!("Herb thinks you are a jerk and refuses to appear.  By the way, \
                                the hash map is: {:?}",
                               *hm);
                  }));

    fly.add_quit_item("Quit");

    fly.display();
}
