extern crate ack;

use std::fs;
use std::io::Read;
use std::path::Path;

use ack::runtime::Ack;

const DIRECTORY: &'static str = "./tests/js";

#[test]
fn run_tests() {
    for entry in fs::read_dir(&Path::new(DIRECTORY)).ok().expect(&format!("Could not find {}", DIRECTORY)) {
        let path = entry.unwrap().path();
        let mut file = fs::File::open(&path).ok().expect(&format!("Could not open {}", path.to_string_lossy()));

        let mut s = String::new();
        file.read_to_string(&mut s).ok().expect(&format!("Could not read {}", path.to_string_lossy()));

        match Ack::create_stdlib().eval(&s) {
            Ok(_) => (),
            Err(e) => panic!("Script failed with error \"{}\"", e.debug_string())
        }
    }
}
