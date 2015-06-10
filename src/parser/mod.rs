mod grammar;
mod complete;

pub use self::grammar::parse;

pub fn is_complete(source: &str) -> bool {
    complete::complete(source).is_ok()
}
