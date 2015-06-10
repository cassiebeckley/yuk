extern crate ack;

#[macro_use]
extern crate maplit;

use std::io;
use std::io::prelude::*;
// use std::fs;

use ack::parser;
use ack::runtime;
use ack::runtime::{Rc};

fn console_log(this: runtime::Value, arguments: Vec<runtime::Value>, global: runtime::Object) -> runtime::Value {
    for value in arguments {
        print!("{} ", value.debug_string());
    }
    println!("");

    runtime::Value::Undefined
}

fn number_prototype_to_string(this: runtime::Value, _: Vec<runtime::Value>, _: runtime::Object) -> runtime::Value {
    match this {
        runtime::Value::Number(n) => runtime::Value::String(n.to_string()),
        _ => panic!("{:?} is not a number!", this)
    }
}

fn string_prototype_to_string(this: runtime::Value, _: Vec<runtime::Value>, _: runtime::Object) -> runtime::Value {
    match this {
        runtime::Value::String(s) => runtime::Value::String(s.to_string()),
        _ => panic!("{:?} is not a string!", this)
    }
}

fn main() {
    // let source = {
    //     let mut f = fs::File::open("foo.js").ok().expect("Could not open foo.js");
    //     let mut s = String::new();
    //     f.read_to_string(&mut s).ok().expect("Could not read foo.js");

    //     s
    // };

    // initialize global object

    let global = runtime::Object::from_map(hashmap!{
        "console".to_string() => runtime::Value::Object(runtime::Object::from_map(hashmap!{
            "log".to_string() => runtime::Value::Function(Rc::new(runtime::Function::Native(console_log)))
        })),
        "Number".to_string() => runtime::Value::Object(runtime::Object::from_map(hashmap!{
            "prototype".to_string() => runtime::Value::Object(runtime::Object::from_map(hashmap!{
                "toString".to_string() => runtime::Value::Function(Rc::new(runtime::Function::Native(number_prototype_to_string)))
            }))
        })),
        "String".to_string() => runtime::Value::Object(runtime::Object::from_map(hashmap!{
            "prototype".to_string() => runtime::Value::Object(runtime::Object::from_map(hashmap!{
                "toString".to_string() => runtime::Value::Function(Rc::new(runtime::Function::Native(string_prototype_to_string)))
            }))
        })),
    });

    // global.set(&"global".to_string(), global.clone());

    loop {
        print!(">>> ");
        io::stdout().flush().unwrap();

        let source = {
            let mut source = String::new();
            io::stdin().read_line(&mut source).unwrap();

            while !parser::is_complete(&source) {
                print!("... ");
                io::stdout().flush().unwrap();
                let mut line = String::new();
                io::stdin().read_line(&mut line).unwrap();
                source = source + &line;
            }

            source
        };

        let parsed = parser::parse(&source);

        println!("");
        println!("AST: {:?}", parsed);

        match parsed {
            Ok(ast) => {
                let result = runtime::eval(&ast, global.clone(), global.clone());
                println!("Result: {:?}", result);
            },
            Err(e) => println!("{:?}", e)
        }
    }
}
