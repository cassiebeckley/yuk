use std::collections::HashMap;
use std::fmt;
pub use std::rc::Rc;
pub use std::cell::RefCell;

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
    Object(Rc<RefCell<Object>>),
    Undefined
}

// TODO: replace all panic!s with JavaScript exception handling

fn apply(function: Value, arguments: Vec<Value>, env: &Object) -> Value {
    match &function {
        &Value::Function(ref f) => f.apply(arguments, env),
        _ => panic!("{:?} is not a function", function)
    }
}

fn lookup<'val>(object: Value, key: &String) -> Value {
    match object {
        Value::Object(ref map) => map.borrow().access(key),
        _ => panic!("{:?} is not an object (TODO: treat functions as objects)", object)
    }
}

fn set(object: Value, key: &String, val: Value) -> Value {
    match object {
        Value::Object(ref map) => {
            map.borrow_mut().insert(key.clone(), val);
            map.borrow().access(key)
        },
        _ => panic!("{:?} is not an object (TODO: treat functions as objects)", object)
    }
}

fn eval_expression_list(expressions: &Vec<ast::Expression>, env: &mut Object) -> Vec<Value> {
    let mut values = vec![];

    for e in expressions {
        let val: Value = eval_expression(e, env);
        values.push(val);
    }

    values
}

fn access_get<'val>(access: &'val ast::Access, env: &'val mut Object) -> Value {
    match access {
        &ast::Access::Member(ref e, ref i) => lookup(eval_expression(e, env), i),
        &ast::Access::Identifier(ref i) => env.access(i)
    }
}

fn access_set<'val>(access: &'val ast::Access, env: &'val mut Object, val: Value) -> Value {
    match access {
        &ast::Access::Member(ref e, ref i) => set(eval_expression(e, env), i, val),
        &ast::Access::Identifier(ref i) => {
            env.insert(i.clone(), val);
            env.access(i)
        }
    }
}

fn eval_expression(expression: &ast::Expression, env: &mut Object) -> Value {
    match expression {
        &ast::Expression::Assignment(ref lhs, ref rhs) => {
            let rhs = eval_expression(rhs, env);
            access_set(lhs, env, rhs)
        },
        &ast::Expression::Call(ref f, ref a) => apply(eval_expression(f, env), eval_expression_list(a, env), env),
        &ast::Expression::Access(ref a) => access_get(a, env),
        // TODO: get rid of clone
        &ast::Expression::Literal(ref l) => l.clone()
    }
}

fn eval_statement(statement: ast::Statement, env: &mut Object) -> Value {
    match statement {
        ast::Statement::Expression(e) => eval_expression(&e, env).clone(),
        ast::Statement::Empty => Value::Undefined
    }
}

pub fn eval(program: ast::Block, env: &mut Object) -> Value {
    let mut last = Value::Undefined;
    for statement in program {
        last = eval_statement(statement, env);
    }

    last
}
