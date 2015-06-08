extern crate ack;

#[macro_use]
extern crate maplit;

use std::io;
use std::io::prelude::*;
// use std::fs;

use ack::parser;
use ack::runtime;
use ack::runtime::{Dictionary, Rc, RefCell};

fn console_log(arguments: Vec<runtime::Value>, _: runtime::Value) -> runtime::Value {
    for value in arguments {
        print!("{:?} ", value);
    }
    println!("");

    runtime::Value::Undefined
}

fn js_exit(_: Vec<runtime::Value>, env: runtime::Value) -> runtime::Value {
    env.set(&"__running".to_string(), runtime::Value::Number(0.0));

    runtime::Value::Undefined
}

fn main() {
    // let source = {
    //     let mut f = fs::File::open("foo.js").ok().expect("Could not open foo.js");
    //     let mut s = String::new();
    //     f.read_to_string(&mut s).ok().expect("Could not read foo.js");

    //     s
    // };

    // initialize global object

    let global = runtime::Value::Object(Rc::new(RefCell::new(runtime::Object::from_map(hashmap!{
        "console".to_string() => runtime::Value::Object(Rc::new(RefCell::new(runtime::Object::from_map(hashmap!{
            "log".to_string() => runtime::Value::Function(Rc::new(runtime::Function::Native(console_log)))
        })))),
        // TODO: be less terrible
        "__running".to_string() => runtime::Value::Number(1.0),
        "exit".to_string() => runtime::Value::Function(Rc::new(runtime::Function::Native(js_exit)))
    }))));

    while let runtime::Value::Number(1.0) = global.get(&"__running".to_string()) {
        print!(">>> ");
        io::stdout().flush().unwrap();

        let source = {
            let mut total = String::new();

            // hacky, but it's just a temporary testing solution
            let mut again = true;

            while again {
                again = false;
                let mut line = String::new();
                io::stdin().read_line(&mut line).unwrap();

                let mut s: Vec<char> = line.trim().chars().collect();
                let last = s.pop();

                match last {
                    Some('\\') => again = true,
                    Some(c) => s.push(c),
                    None => ()
                }

                s.push('\n');

                for c in s {
                    total.push(c)
                }
            }

            total
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
