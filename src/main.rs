extern crate ack;

use std::io;
use std::io::prelude::*;
// use std::fs;

use ack::lexer::Lexer;

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
            println!("{}", token.to_string());
        }
    }
}
