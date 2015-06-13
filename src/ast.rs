pub use super::interpret::{Value, UserFunction};

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
    Function(Identifier, UserFunction)
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Positive,
    Negative
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide
}

#[derive(Debug, Clone)]
pub enum Expression {
    Assignment(Access, Box<Expression>),
    Access(Access),
    Call(Box<Expression>, ExpressionList),
    Literal(Value),
    Function(UserFunction),
    Unary(UnaryOp, Box<Expression>),
    Binary(BinaryOp, Box<Expression>, Box<Expression>),
    This
}

#[derive(Debug, Clone)]
pub enum Access {
    Member(Box<Expression>, Accessor),
    Identifier(Identifier)
}

#[derive(Debug, Clone)]
pub enum Accessor {
    Identifier(Identifier),
    Expression(Box<Expression>)
}

pub type Identifier = String;
