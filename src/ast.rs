pub use super::interpret::{Value, Function};

pub type Block = Vec<Statement>;

pub type ExpressionList = Vec<Expression>;

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Expression),
    Declaration(Declaration),
    Throw(Expression),
    Empty
}

#[derive(Debug, Clone)]
pub enum Declaration {
    Variable(Identifier, Option<Expression>),
    Function(Identifier, Function)
}

#[derive(Debug, Clone)]
pub enum Expression {
    Assignment(Access, Box<Expression>),
    Access(Access),
    Call(Box<Expression>, ExpressionList),
    Literal(Value),
    This
}

#[derive(Debug, Clone)]
pub enum Access {
    Member(Box<Expression>, Identifier),
    Identifier(Identifier)
}

pub type Identifier = String;
