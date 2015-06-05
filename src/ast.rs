pub use super::runtime::Value;

pub type Block = Vec<Statement>;
pub type ExpressionList = Vec<Expression>;

#[derive(Debug)]
pub enum Statement {
    Expression(Expression),
    Empty
}

#[derive(Debug)]
pub enum Expression {
    Call(Box<Expression>, Vec<Expression>),
    Member(Box<Expression>, Identifier),
    Identifier(Identifier),
    Literal(Value)
}

pub type Identifier = String;
