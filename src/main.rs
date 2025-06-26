use rs_hash::HashMap;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn hash<T: Hash>(key: &T) -> u32 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish() as u32
}

fn main() {
    let mut map = HashMap::with_capacity(1, hash);

    map.insert("me", "Max");
    map.insert("us", "World");

    let value = map.get("us").expect("Expected value to be present");

    println!("Hello, {}!", value);

    map.remove("me");

    match map.get("me") {
        Some(value) => println!("Hello back, {}", value),
        None => println!("Removed above dumbass..."),
    };
}
