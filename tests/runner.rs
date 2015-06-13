extern crate ack;

use std::fs;
use std::io::Read;
use std::path::Path;

use ack::runtime::Ack;
use ack::interpret::{Value, Function, JSResult, Context, throw_string};

const DIRECTORY: &'static str = "./tests/js/";
const COUNT: &'static str = "__assert_eq_call_count";

// Use strict equals, keeping track of the number of calls
fn assert_eq(arguments: Vec<Value>, context: Context) -> JSResult {
    // increment the call count
    let calls = match context.global.get(COUNT) {
        Ok(Value::Number(n)) => n,
        _ => 0.0
    };
    context.global.set(COUNT, Value::Number(calls + 1.0)).unwrap_or(Value::Undefined);

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

        let result = ack.eval(&s);
        println!("{} assertions called", match ack.global.get(COUNT) {Ok(Value::Number(n)) => n, _ => 0.0});

        match result {
            Ok(_) => (),
            Err(e) => panic!("{} failed with error \"{}\"", path.to_string_lossy(), e.debug_string())
        }
    }
}
