extern crate regex;
use self::regex::Regex;

#[derive(Clone, Debug, PartialEq)]
pub enum TokenType {
    Whitespace,
    Identifier,
    Integer,
    // Keyword,
    Symbol,
}

#[derive(Debug, PartialEq)]
pub struct Token<'a> {
    pub which: TokenType,
    pub value: &'a str
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
        let symbol = Regex::new(r"^[.();,]").unwrap();

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

#[cfg(test)]
mod tests {
    use super::Lexer;
    use super::Token;
    use super::TokenType;

    #[test]
    fn whitespace() {
        let lexed: Vec<_> = Lexer::new(" \n\r\t").collect();
        assert_eq!(lexed.len(), 1);
        assert_eq!(lexed[0], Token {which: TokenType::Whitespace, value: " \n\r\t"});
    }

    #[test]
    fn identifier() {
        let lexed: Vec<_> = Lexer::new("hello_there").collect();
        assert_eq!(lexed.len(), 1);
        assert_eq!(lexed[0], Token {which: TokenType::Identifier, value: "hello_there"});
    }

    #[test]
    fn integer() {
        let lexed: Vec<_> = Lexer::new("1234").collect();
        assert_eq!(lexed.len(), 1);
        assert_eq!(lexed[0], Token {which: TokenType::Integer, value: "1234"});
    }

    #[test]
    fn symbol() {
        for s in [".", "(", ")"].iter() {
            let lexed: Vec<_> = Lexer::new(s).collect();
            assert_eq!(lexed.len(), 1);
            assert_eq!(lexed[0], Token {which: TokenType::Symbol, value: s});
        }
    }
}
