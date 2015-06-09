use std::collections::HashMap;
use std::{fmt, iter};
pub use std::rc::Rc;
pub use std::cell::RefCell;

use super::ast;

pub trait Dictionary {
    fn get(&self, &String) -> Value;
    fn set(&mut self, &String, Value) -> Value;
}

#[derive(Debug, Clone)]
pub struct Object {
    values: HashMap<String, Value>,
    prototype: Option<Rc<RefCell<Object>>>
}

impl Object {
    pub fn new() -> Object {
        Object {
            values: HashMap::new(),
            prototype: None
        }
    }

    pub fn create(proto: Rc<RefCell<Object>>) -> Object {
        let proto = proto.clone();

        Object {
            values: HashMap::new(),
            prototype: Some(proto)
        }
    }

    pub fn from_map(map: HashMap<String, Value>) -> Object {
        Object {
            values: map,
            prototype: None
        }
    }

    fn outer_set(&mut self, key: &String, val: Value) -> Value {
        if self.values.contains_key(key) {
            self.values.insert(key.clone(), val.clone());
        } else if let Some(ref proto) = self.prototype {
            proto.borrow_mut().outer_set(key, val.clone());
        } else {
            self.values.insert(key.clone(), val.clone());
        }

        val
    }
}

impl Dictionary for Object {
    fn get(&self, key: &String) -> Value {
        match self.values.get(key) {
            Some(v) => v.clone(),
            None => match self.prototype {
                Some(ref proto) => proto.borrow().get(key),
                None => Value::Undefined
            }
        }
    }

    fn set(&mut self, key: &String, val: Value) -> Value {
        self.values.insert(key.clone(), val.clone());
        val
    }
}

trait ToValue {
    fn to_value(&self) -> Value;
}

impl <'a>ToValue for Option<&'a Value> {
    fn to_value(&self) -> Value {
        match self {
            &Some(val) => val.clone(),
            &None => Value::Undefined
        }
    }
}

pub enum Function {
    Native(fn(Vec<Value>, Value) -> Value),
    User {id: Option<ast::Identifier>, parameters: Vec<String>, body: ast::Block, source: String}
}

impl fmt::Debug for Function {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Function::Native(_) => fmt.write_str("Native([native code])"),
            &Function::User {
                id: ref i,
                parameters: ref p,
                body: ref b,
                source: _
            } => fmt.write_fmt(format_args!("User {{ id: {:?}, parameters: {:?}, body: {:?} }}", i, p, b))
        }
    }
}

impl Function {
    fn apply(&self, arguments: Vec<Value>, global: Value) -> Value {
        match self {
            &Function::Native(f) => f(arguments, global),
            &Function::User {id: _, parameters: ref p, body: ref b, source: _} => {
                let glob = match global.clone() {
                    Value::Object(o) => o,
                    _ => panic!("{:?} is not an object", global)
                };

                let inner_env = Value::Object(Rc::new(RefCell::new(Object::create(glob))));
                let undef = Value::Undefined;
                for (argument, parameter) in arguments.iter().chain(iter::repeat(&undef)).zip(p) {
                    inner_env.set(parameter, argument.clone());
                }

                eval(b, inner_env.clone(), global);

                Value::Undefined
            }
        }
    }
}

#[derive(Debug,Clone)]
pub enum Value {
    Number(f64),
    Function(Rc<Function>),
    Object(Rc<RefCell<Object>>),
    Null,
    Undefined
}

impl Value {
    pub fn get(&self, key: &String) -> Value {
        match self {
            &Value::Object(ref map) => map.borrow_mut().get(key),
            _ => panic!("{:?} is not an object (TODO: treat functions as objects)", self)
        }
    }

    pub fn set(&self, key: &String, val: Value) -> Value {
        match self {
            &Value::Object(ref map) => map.borrow_mut().set(key, val),
            _ => panic!("{:?} is not an object (TODO: treat functions as objects)", self)
        }
    }

    pub fn outer_set(&self, key: &String, val: Value) -> Value {
        match self {
            &Value::Object(ref map) => map.borrow_mut().outer_set(key, val),
            _ => panic!("{:?} is not an object (TODO: treat functions as objects)", self)
        }
    }
}

// TODO: replace all panic!s with JavaScript exception handling

fn apply(function: Value, arguments: Vec<Value>, env: Value) -> Value {
    match &function {
        &Value::Function(ref f) => f.apply(arguments, env),
        _ => panic!("{:?} is not a function", function)
    }
}

fn eval_expression_list(expressions: &Vec<ast::Expression>, local: Value, global: Value) -> Vec<Value> {
    let mut values = vec![];

    for e in expressions {
        let val: Value = eval_expression(e, local.clone(), global.clone());
        values.push(val);
    }

    values
}

fn access_get(access: &ast::Access, local: Value, global: Value) -> Value {
    match access {
        &ast::Access::Member(ref e, ref i) => eval_expression(e, local, global).get(i),
        &ast::Access::Identifier(ref i) => local.get(i)
    }
}

fn access_set(access: &ast::Access, local: Value, global: Value, val: Value) -> Value {
    match access {
        &ast::Access::Member(ref e, ref i) => eval_expression(e, local, global).set(i, val),
        &ast::Access::Identifier(ref i) => {
            local.outer_set(i, val);
            local.get(i)
        }
    }
}

fn eval_expression(expression: &ast::Expression, local: Value, global: Value) -> Value {
    match expression {
        &ast::Expression::Assignment(ref lhs, ref rhs) => {
            let rhs = eval_expression(rhs, local.clone(), global.clone());
            access_set(lhs, local, global, rhs)
        },
        &ast::Expression::Call(ref f, ref a) => {
            let func = eval_expression(f, local.clone(), global.clone()); 
            let args = eval_expression_list(a, local.clone(), global.clone());
            apply(func, args, global)
        },
        &ast::Expression::Access(ref a) => access_get(a, local, global),
        // TODO: get rid of clone
        &ast::Expression::Literal(ref l) => l.clone()
    }
}

fn eval_statement(statement: &ast::Statement, local: Value, global: Value) -> Value {
    match statement {
        &ast::Statement::Expression(ref e) => eval_expression(e, local, global).clone(),
        &ast::Statement::Empty => Value::Undefined
    }
}

pub fn eval(program: &ast::Block, local: Value, global: Value) -> Value {
    let mut last = Value::Undefined;
    for statement in program {
        last = eval_statement(statement, local.clone(), global.clone());
    }

    last
}
