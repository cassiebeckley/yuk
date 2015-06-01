extern crate regex;
use self::regex::Regex;

#[derive(Clone)]
enum TokenType {
    Whitespace,
    Identifier,
    Integer,
    // Keyword,
    Symbol,
}

impl ToString for TokenType {
    fn to_string(&self) -> String {
        match *self {
            TokenType::Whitespace => "Whitespace",
            TokenType::Identifier => "Identifier",
            TokenType::Integer => "Integer",
            // TokenType::Keyword => "Keyword",
            TokenType::Symbol => "Symbol",
        }.to_string()
    }
}

pub struct Token<'a> {
    which: TokenType,
    value: &'a str
}

impl <'a>ToString for Token<'a> {
    fn to_string(&self) -> String {
        self.which.to_string() + ": " + &self.value
    }
}

pub struct Lexer<'a> {
    source: &'a str
}

impl <'a>Lexer<'a> {
    pub fn new(s: &str) -> Lexer {
        Lexer {source: s}
    }
}

impl <'a>Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
        let whitespace = Regex::new(r"^\s+").unwrap();
        let identifier = Regex::new(r"^[a-zA-Z_]+").unwrap();
        let integer = Regex::new(r"^\d+").unwrap();
        let symbol = Regex::new(r"^[.()]").unwrap();

        let patterns = vec![(whitespace, TokenType::Whitespace), (identifier, TokenType::Identifier), (integer, TokenType::Integer), (symbol, TokenType::Symbol)];

        for e in &patterns {
            let &(ref p, ref t) = e;
            let cs = p.captures(self.source);

            if let Some(captures) = cs {
                let split: Vec<_> = p.splitn(self.source, 2).collect();
                self.source = split[1];

                return Some(Token {which: t.clone(), value: captures.at(0).unwrap()});
            }
        }

        return None;
    }
}
