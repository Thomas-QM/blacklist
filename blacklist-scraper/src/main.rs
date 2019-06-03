extern crate blacklist_lib;
use blacklist_lib::*;

extern crate serde;
extern crate serde_json;

extern crate dotenv;
extern crate rayon;
extern crate reqwest;

use std::fs;
use std::env;

mod syntaxes;

fn main() {
    dotenv::dotenv().unwrap();

    let mut append = false;
    let mut ext_only = false;

    for arg in env::args() {
        if arg == "ext-only" { ext_only = true }
        else if arg == "append" { append = true }
    }

    let mut bl = if append {
            load()
        } else {
            Vec::new()
        };

    println!("Loaded blacklist");

    let bl_ext: Vec<BlacklistExtItem> = serde_json::from_str(&fs::read_to_string("./blacklist-ext.json").unwrap()).unwrap();
    bl.extend(bl_ext.iter().map(BlacklistExtItem::to_bi));
    
    println!("Loaded blacklist-ext");

    if !ext_only {
        syntaxes::scrape(&mut bl);
    }

    println!("Cleaning data...");
    bl.sort_unstable();
	bl.dedup();

    println!("Writing to file...");
    write(&bl);

    println!("Finished: {}", bl.len());
}