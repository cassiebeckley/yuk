use super::lexer;
use super::ast;

#[derive(Debug)]
enum Node {
    Block(ast::Block),
    Statement(ast::Statement),
    Expression(ast::Expression),
    Identifier(ast::Identifier),
    Symbol(String)
}

enum Pattern<'a> {
    Block,
    Statement,
    Expression,
    Identifier,
    Symbol(&'a str)
}

fn equal(p: &Pattern, n: &Node) -> bool {
    match p {
        &Pattern::Block => match n {
            &Node::Block(_) => true,
            _ => false
        },
        &Pattern::Statement => match n {
            &Node::Statement(_) => true,
            _ => false
        },
        &Pattern::Expression => match n {
            &Node::Expression(_) => true,
            _ => false
        },
        &Pattern::Identifier => match n {
            &Node::Identifier(_) => true,
            _ => false
        },
        &Pattern::Symbol(ps) => match n {
            &Node::Symbol(ref ns) => ps == ns,
            _ => false
        }
    }
}

fn reduce<F: Fn(Vec<Node>) -> Node>(stack: &mut Vec<Node>, pattern: &[Pattern], action: F) -> bool {
    if pattern.len() > stack.len() {
        return false;
    }

    let mut matches: Vec<Node> = vec![];

    {
        let (_, end) = stack.split_at(stack.len() - pattern.len());

        for (p, n) in pattern.iter().zip(end) {
            if !equal(p, n) {
                return false;
            }
        }
    }

    for _ in pattern {
        matches.push(stack.pop().unwrap());
    }

    stack.push(action(matches));
    return true;
}

// Implement shift-reduce parser
pub fn parse(source: &str) -> Option<ast::Block> {
    let lex = lexer::Lexer::new(source);
    let mut stack: Vec<Node> = Vec::new();

    for token in lex {
        let node = match token {
            lexer::Token {which: lexer::TokenType::Whitespace, value: _} => continue,
            lexer::Token {which: lexer::TokenType::Identifier, value: id} => Node::Identifier(id.to_string()),
            lexer::Token {which: lexer::TokenType::Integer, value: num} => Node::Expression(ast::Expression::Literal(ast::Value::Number(num.parse::<f64>().unwrap()))),
            lexer::Token {which: lexer::TokenType::Symbol, value: sym} => Node::Symbol(sym.to_string()),
        };

        stack.push(node);

        let mut reduced = true;

        // I have no idea how this works; which is odd considering that I wrote it
        /*while reduced*/ {
            reduced = false;

            reduced |= reduce(&mut stack, &[Pattern::Identifier, Pattern::Symbol("."), Pattern::Identifier], |mut matches| {
                let (m0, _, m2) = (matches.pop().unwrap(), matches.pop(), matches.pop().unwrap());

                let object = match m0 {
                    Node::Identifier(id) => id,
                    _ => unreachable!()
                };

                let member = match m2 {
                    Node::Identifier(id) => id,
                    _ => unreachable!()
                };

                Node::Expression(
                    ast::Expression::Member(Box::new(ast::Expression::Identifier(object)), member)
                )
            });

            reduced |= reduce(&mut stack, &[Pattern::Expression, Pattern::Symbol("."), Pattern::Identifier], |mut matches| {
                let (m0, _, m2) = (matches.pop().unwrap(), matches.pop().unwrap(), matches.pop().unwrap());

                let object = match m0 {
                    Node::Expression(exp) => exp,
                    _ => unreachable!()
                };

                let member = match m2 {
                    Node::Identifier(id) => id,
                    _ => unreachable!()
                };

                Node::Expression(
                    ast::Expression::Member(Box::new(object), member)
                )
            });

            reduced |= reduce(&mut stack, &[Pattern::Expression, Pattern::Symbol("("), Pattern::Expression, Pattern::Symbol(")")], |mut matches| {
                let (m0, _, m2, _) = (matches.pop().unwrap(), matches.pop().unwrap(), matches.pop().unwrap(), matches.pop().unwrap());

                let function = match m0 {
                    Node::Expression(exp) => exp,
                    _ => unreachable!()
                };

                let argument = match m2 {
                    Node::Expression(exp) => exp,
                    _ => unreachable!()
                };

                Node::Expression(
                    ast::Expression::Call(Box::new(function), vec![argument])
                )
            });

            reduced |= reduce(&mut stack, &[Pattern::Identifier, Pattern::Symbol("("), Pattern::Expression, Pattern::Symbol(")")], |mut matches| {
                let (m0, _, m2, _) = (matches.pop().unwrap(), matches.pop().unwrap(), matches.pop().unwrap(), matches.pop().unwrap());

                let function_id = match m0 {
                    Node::Identifier(id) => id,
                    _ => unreachable!()
                };

                let argument = match m2 {
                    Node::Expression(exp) => exp,
                    _ => unreachable!()
                };

                Node::Expression(
                    ast::Expression::Call(Box::new(ast::Expression::Identifier(function_id)), vec![argument])
                )
            });

            reduced |= reduce(&mut stack, &[Pattern::Expression, Pattern::Symbol(";")], |mut matches| {
                let (m0, _) = (matches.pop().unwrap(), matches.pop().unwrap());

                let expression = match m0 {
                    Node::Expression(exp) => exp,
                    _ => unreachable!()
                };

                Node::Statement(
                    ast::Statement::Expression(expression)
                )
            });

            reduced |= reduce(&mut stack, &[Pattern::Block, Pattern::Statement], |mut matches| {
                let (m0, m1) = (matches.pop().unwrap(), matches.pop().unwrap());

                let mut block = match m0 {
                    Node::Block(p) => p,
                    _ => unreachable!()
                };

                let statement = match m1 {
                    Node::Statement(s) => s,
                    _ => unreachable!()
                };

                block.push(statement);

                Node::Block(block)
            });

            reduced |= reduce(&mut stack, &[Pattern::Statement], |mut matches| {
                let m0 = matches.pop().unwrap();

                let statement = match m0 {
                    Node::Statement(s) => s,
                    _ => unreachable!()
                };

                Node::Block(vec![statement])
            });
        }
    }

    println!("parse stack: {:?}", stack);

    if let Some(Node::Block(p)) = stack.pop() {
        Some(p)
    } else {
        None
    }
}
