use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::{fmt, iter};
pub use std::rc::Rc;
pub use std::cell::RefCell;

use super::ast;

pub trait Dictionary {
    fn get(&mut self, &String) -> Value;
    fn set(&mut self, &String, Value) -> Value;
}

#[derive(Debug, Clone)]
pub struct Object {
    values: HashMap<String, Value>,
    prototype: Option<Box<Object>>
}

impl Object {
    pub fn new() -> Object {
        Object {
            values: HashMap::new(),
            prototype: None
        }
    }

    pub fn create(proto: &Object) -> Object {
        let proto: Object = proto.clone();

        Object {
            values: HashMap::new(),
            prototype: Some(Box::new(proto))
        }
    }

    pub fn from_map(map: HashMap<String, Value>) -> Object {
        Object {
            values: map,
            prototype: None
        }
    }

    fn entry(&mut self, key: &String) -> Entry<String, Value> {
        match self.values.entry(key.clone()) {
            o@Entry::Occupied(_) => o,
            v@Entry::Vacant(_) => match self.prototype {
                Some(ref mut proto) => {
                    proto.entry(key)
                },
                None => v
            }
        }
    }
}

impl Dictionary for Object {
    fn get(&mut self, key: &String) -> Value {
        match self.entry(key) {
            Entry::Occupied(o) => o.get().clone(),
            Entry::Vacant(_) => Value::Undefined
        }
    }

    fn set(&mut self, key: &String, val: Value) -> Value {
        match self.entry(key) {
            Entry::Occupied(ref mut o) => {
                o.insert(val.clone());
                val
            },
            Entry::Vacant(v) => {
                v.insert(val.clone());
                val
            }
        }
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
            &Function::User {id: ref i, parameters: ref p, body: ref b, source: _} => {
                let glob = match global.clone() {
                    Value::Object(o) => o,
                    _ => panic!("{:?} is not an object", global)
                };

                let inner_env = Value::Object(Rc::new(RefCell::new(Object::create(&glob.borrow()))));
                let undef = Value::Undefined;
                for (argument, parameter) in arguments.iter().chain(iter::repeat(&undef)).zip(p) {
                    inner_env.set(parameter, argument.clone());
                }

                println!("Function {:?} called with {:?}", i, inner_env);
                eval(b, inner_env, global)
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
            &Value::Object(ref map) => {
                map.borrow_mut().set(key, val);
                map.borrow_mut().get(key)
            },
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

fn eval_expression_list(expressions: &Vec<ast::Expression>, env: Value) -> Vec<Value> {
    let mut values = vec![];

    for e in expressions {
        let val: Value = eval_expression(e, env.clone());
        values.push(val);
    }

    values
}

fn access_get(access: &ast::Access, env: Value) -> Value {
    match access {
        &ast::Access::Member(ref e, ref i) => eval_expression(e, env).get(i),
        &ast::Access::Identifier(ref i) => env.get(i)
    }
}

fn access_set(access: &ast::Access, env: Value, val: Value) -> Value {
    match access {
        &ast::Access::Member(ref e, ref i) => eval_expression(e, env).set(i, val),
        &ast::Access::Identifier(ref i) => {
            env.set(i, val);
            env.get(i)
        }
    }
}

fn eval_expression(expression: &ast::Expression, env: Value) -> Value {
    match expression {
        &ast::Expression::Assignment(ref lhs, ref rhs) => {
            let rhs = eval_expression(rhs, env.clone());
            access_set(lhs, env, rhs)
        },
        &ast::Expression::Call(ref f, ref a) => apply(eval_expression(f, env.clone()), eval_expression_list(a, env.clone()), env),
        &ast::Expression::Access(ref a) => access_get(a, env),
        // TODO: get rid of clone
        &ast::Expression::Literal(ref l) => l.clone()
    }
}

fn eval_statement(statement: &ast::Statement, local: Value, global: Value) -> Value {
    let local = match local {
        Value::Object(_) => local,
        _ => panic!("local must be an object")
    };

    match statement {
        &ast::Statement::Expression(ref e) => eval_expression(e, local).clone(),
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
