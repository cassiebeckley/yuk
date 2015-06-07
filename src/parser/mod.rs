mod grammar;

use super::ast;

pub fn parse(source: &str) -> grammar::ParseResult<ast::Block> {
    grammar::parse(source)
}
