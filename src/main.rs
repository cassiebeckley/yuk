extern crate ack;

use std::io;
use std::io::prelude::*;
// use std::fs;

use ack::parser;
use ack::runtime::Ack;

fn main() {
    // let source = {
    //     let mut f = fs::File::open("foo.js").ok().expect("Could not open foo.js");
    //     let mut s = String::new();
    //     f.read_to_string(&mut s).ok().expect("Could not read foo.js");

    //     s
    // };

    let mut ack = Ack::new();

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

        println!("Result: {:?}", ack.eval(&source));
    }
}
