extern crate ack;

use std::fs;
use std::io::Read;
use std::path::Path;

use ack::runtime::Ack;
use ack::interpret::{Value, Function, JSResult, Context, throw_string};

const DIRECTORY: &'static str = "./tests/js";

// Use strict equals
fn assert_eq(arguments: Vec<Value>, _: Context) -> JSResult {
    let undefined = Value::Undefined;

    let (a, b) = (arguments.get(0).unwrap_or(&undefined), arguments.get(1).unwrap_or(&undefined));

    if a.strict_equals(b) {
        Ok(Value::Undefined)
    } else {
        throw_string(format!("{} !== {}", a.debug_string(), b.debug_string()))
    }
}

#[test]
fn run_tests() {
    for entry in fs::read_dir(&Path::new(DIRECTORY)).ok().expect(&format!("Could not find {}", DIRECTORY)) {
        let path = entry.unwrap().path();
        let mut file = fs::File::open(&path).ok().expect(&format!("Could not open {}", path.to_string_lossy()));

        let mut s = String::new();
        file.read_to_string(&mut s).ok().expect(&format!("Could not read {}", path.to_string_lossy()));

        let mut ack = Ack::create_stdlib();
        ack.global.set("assert_eq", Value::from_function(Function::Native(assert_eq), ack.global.clone())).unwrap();

        match ack.eval(&s) {
            Ok(_) => (),
            Err(e) => panic!("Script failed with error \"{}\"", e.debug_string())
        }
    }
}
