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
    Assignment(Access, Box<Expression>),
    Access(Access),
    Call(Box<Expression>, ExpressionList),
    Literal(Value)
}

#[derive(Debug)]
pub enum Access {
    Member(Box<Expression>, Identifier),
    Identifier(Identifier)
}

pub type Identifier = String;
