use std::collections::HashMap;
use std::fmt;
pub use std::rc::Rc;

use super::ast;

pub type Object = HashMap<String, Value>;

trait Access {
    fn access(&self, key: &String) -> Value;
}

impl Access for Object {
    fn access(&self, key: &String) -> Value {
        match self.get(key) {
            Some(val) => val.clone(),
            None => Value::Undefined
        }
    }
}

pub enum Function {
    Native(fn(Vec<Value>, &Object) -> Value)
}

impl fmt::Debug for Function {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Function::Native(_) => fmt.write_str("Native([native code])")
        }
    }
}

impl Function {
    fn apply(&self, arguments: Vec<Value>, env: &Object) -> Value {
        match self {
            &Function::Native(f) => f(arguments, env)
        }
    }
}

#[derive(Debug,Clone)]
pub enum Value {
    Number(f64),
    Function(Rc<Function>),
    Object(Object),
    Undefined
}

// TODO: replace all panic!s with JavaScript exception handling

fn apply(function: Value, arguments: Vec<Value>, env: &Object) -> Value {
    let function = match function {
        Value::Function(f) => f,
        _ => panic!("{:?} is not a function", function)
    };

    function.apply(arguments, env)
}

fn lookup(object: Value, key: &String) -> Value {
    let object = match object {
        Value::Object(map) => map,
        _ => panic!("{:?} is not an object (TODO: treat functions as objects)", object)
    };

    object.access(key)
}

fn eval_expression_list(expressions: &Vec<ast::Expression>, env: &Object) -> Vec<Value> {
    let mut values = vec![];

    for e in expressions {
        values.push(eval_expression(e, env));
    }

    values
}

fn eval_expression(expression: &ast::Expression, env: &Object) -> Value {
    match expression {
        &ast::Expression::Call(ref f, ref a) => apply(eval_expression(f, env), eval_expression_list(a, env), env),
        &ast::Expression::Member(ref e, ref i) => lookup(eval_expression(e, env), i),
        &ast::Expression::Identifier(ref i) => env.access(i),
        // TODO: get rid of clone
        &ast::Expression::Literal(ref l) => l.clone()
    }
}

fn eval_statement(statement: ast::Statement, env: &Object) -> Value {
    match statement {
        ast::Statement::Expression(e) => eval_expression(&e, env),
        ast::Statement::Empty => Value::Undefined
    }
}

pub fn eval(program: ast::Block, env: Object) -> Value {
    let mut last = Value::Undefined;
    for statement in program {
        last = eval_statement(statement, &env);
    }

    last
}
