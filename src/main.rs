extern crate ack;
extern crate libc;
extern crate term;

use std::{io, env, fs};
use std::io::Write;

use ack::parser;
use ack::runtime::Ack;

fn is_interactive() -> bool {
    (unsafe { libc::isatty(libc::STDIN_FILENO as i32) }) != 0
}

fn start_repl() {
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

        match ack.eval(&source) {
            Ok(result) => {
                let mut t = term::stdout().unwrap();

                t.fg(term::color::BRIGHT_BLUE).unwrap();
                writeln!(t, "{}", result.debug_string()).unwrap();

                t.reset().unwrap();
            },
            Err(e) => {
                let mut t = term::stderr().unwrap();

                t.fg(term::color::BRIGHT_RED).unwrap();
                writeln!(t, "{}", e.debug_string()).unwrap();

                t.reset().unwrap();
            }
        }
    }
}

fn run_script<T: io::Read>(mut file: T) -> bool {
    let source = {
        let mut s = String::new();
        file.read_to_string(&mut s).ok().expect("Could not read file");

        s
    };

    let result = Ack::new().eval(&source);

    if let &Err(ref e) = &result {
        let mut t = term::stderr().unwrap();

        t.fg(term::color::BRIGHT_RED).unwrap();
        writeln!(t, "{}", e.debug_string()).unwrap();

        t.reset().unwrap();
    }

    result.is_ok()
}

fn main() {
    if let Some(filename) = env::args().nth(1) {
        let file = fs::File::open(&filename).ok().expect(&format!("Could not open {}", filename));
        run_script(file);
    } else if is_interactive() {
        start_repl();
    } else {
        run_script(io::stdin());
    }
}
