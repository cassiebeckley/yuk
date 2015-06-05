extern crate ack;

use std::io;
use std::io::prelude::*;
// use std::fs;

use ack::lexer::Lexer;
use ack::parser;
use ack::runtime;

fn console_log(arguments: Vec<runtime::Value>, _: &runtime::Object) -> runtime::Value {
    for value in arguments {
        print!("{:?} ", value);
    }
    println!("");

    runtime::Value::Undefined
}

fn main() {
    // let source = {
    //     let mut f = fs::File::open("foo.js").ok().expect("Could not open foo.js");
    //     let mut s = String::new();
    //     f.read_to_string(&mut s).ok().expect("Could not read foo.js");

    //     s
    // };

    loop {
        print!(">>> ");
        io::stdout().flush().unwrap();

        let source = {
            let mut s = String::new();
            io::stdin().read_line(&mut s).unwrap();
            s
        };

        for token in Lexer::new(&source) {
            // println!("{}", token.to_string());
            println!("{:?}", token);
        }

        println!("");

        let parsed = parser::parse(&source);
        println!("AST: {:?}", parsed);

        if let Some(ast) = parsed {
            let mut global = runtime::Object::new();
            let mut console = runtime::Object::new();
            console.insert("log".to_string(), runtime::Value::Function(runtime::Rc::new(runtime::Function::Native(console_log))));
            global.insert("console".to_string(), runtime::Value::Object(console));
            let result = runtime::eval(ast, global);
            println!("Result: {:?}", result);
        }
    }
}
