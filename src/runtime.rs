use std::collections::HashMap;
use std::{fmt, iter};
pub use std::rc::Rc;
pub use std::cell::RefCell;

use super::ast;

#[derive(Debug, Clone)]
pub struct PlainObject {
    values: HashMap<String, Value>,
    prototype: Object
}

impl PlainObject {
    pub fn new() -> PlainObject {
        PlainObject {
            values: HashMap::new(),
            prototype: Object::Null
        }
    }

    pub fn create(proto: Object) -> PlainObject {
        let proto = proto.clone();

        PlainObject {
            values: HashMap::new(),
            prototype: proto
        }
    }

    pub fn from_map(map: HashMap<String, Value>) -> PlainObject {
        PlainObject {
            values: map,
            prototype: Object::Null
        }
    }

    fn outer_set(&mut self, key: &str, val: Value) -> Value {
        let key = key.to_string();
        if self.values.contains_key(&key) {
            self.values.insert(key, val.clone());
        } else if let Object::Plain(ref proto) = self.prototype {
            proto.borrow_mut().outer_set(&key, val.clone());
        } else {
            self.values.insert(key, val.clone());
        }

        val
    }

    fn get(&self, key: &str) -> Value {
        match self.values.get(&key.to_string()) {
            Some(v) => v.clone(),
            None => match self.prototype {
                Object::Plain(ref proto) => proto.borrow().get(key),
                Object::Null => Value::Undefined
            }
        }
    }

    fn set(&mut self, key: &str, val: Value) -> Value {
        self.values.insert(key.to_string(), val.clone());
        val
    }

    fn debug_string(&self) -> String {
        let middle: String = self.values.iter()
            .map(|(key, value)| "\"".to_string() + key + "\": " + &value.debug_string())
            .fold("".to_string(), |result, next| if result.len() > 0 {result + ", " + &next} else {next});

        "{".to_string() + &middle + "}"
    }
}

#[derive(Debug, Clone)]
pub enum Object {
    Plain(Rc<RefCell<PlainObject>>),
    Null
}

impl Object {
    pub fn from_map(obj: HashMap<String, Value>) -> Object {
        Object::Plain(Rc::new(RefCell::new(PlainObject::from_map(obj))))
    }

    fn outer_set(&self, key: &str, val: Value) -> Value {
        match self {
            &Object::Plain(ref obj) => obj.borrow_mut().outer_set(key, val),
            &Object::Null => panic!("null has no properties")
        }
    }

    pub fn get(&self, key: &str) -> Value {
        match self {
            &Object::Plain(ref obj) => obj.borrow().get(key),
            &Object::Null => panic!("null has no properties")
        }
    }

    pub fn set(&self, key: &str, val: Value) -> Value {
        match self {
            &Object::Plain(ref obj) => obj.borrow_mut().set(key, val),
            &Object::Null => panic!("null has no properties")
        }
    }

    fn debug_string(&self) -> String {
        match self {
            &Object::Plain(ref o) => o.borrow().debug_string(),
            &Object::Null => "null".to_string()
        }
    }
}

#[derive(Clone)]
pub enum Function {
    Native(Rc<fn(Value, Vec<Value>, Object) -> Value>),
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
    fn apply(&self, this: Value, arguments: Vec<Value>, global: Object) -> Value {
        match self {
            &Function::Native(ref f) => f(this, arguments, global),
            &Function::User {id: _, parameters: ref p, body: ref b, source: _} => {
                // TODO: add this binding
                let inner_env = Object::Plain(Rc::new(RefCell::new(PlainObject::create(global.clone()))));
                let undef = Value::Undefined;
                for (argument, parameter) in arguments.iter().chain(iter::repeat(&undef)).zip(p) {
                    inner_env.set(parameter, argument.clone());
                }

                eval(b, inner_env.clone(), global);

                Value::Undefined
            }
        }
    }

    fn debug_string(&self) -> String {
        match self {
            &Function::Native(_) => "function() {\n    [native code]\n}".to_string(),
            &Function::User {id: Some(ref id), parameters: _, body: _, source: _} => format!("function {}()", id),
            &Function::User {id: None, parameters: _, body: _, source: _} => "function()".to_string(),
        }
    }
}

#[derive(Debug,Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Function(Function),
    Object(Object),
    Undefined
}

impl Value {
    pub fn get(&self, key: &str, global: Object) -> Value {
        match self {
            &Value::Number(_) => global.clone().get("Number").get("prototype", global.clone()).get(key, global),
            &Value::String(_) => global.clone().get("String").get("prototype", global.clone()).get(key, global),
            &Value::Object(ref obj) => obj.get(key),
            _ => panic!("{:?} is not an object (TODO: treat functions as objects)", self)
        }
    }

    pub fn set(&self, key: &str, val: Value) -> Value {
        match self {
            &Value::Object(ref obj) => obj.set(key, val),
            _ => panic!("{:?} is not an object (TODO: treat functions as objects)", self)
        }
    }

    pub fn outer_set(&self, key: &str, val: Value) -> Value {
        match self {
            &Value::Object(ref obj) => obj.outer_set(key, val),
            _ => panic!("{:?} is not an object (TODO: treat functions as objects)", self)
        }
    }

    pub fn debug_string(&self) -> String {
        match self {
            &Value::Number(n) => n.to_string(),
            &Value::String(ref s) => s.to_string(),
            &Value::Function(ref f) => f.debug_string(),
            &Value::Object(ref o) => o.debug_string(),
            &Value::Undefined => "undefined".to_string(),
        }
    }
}

// TODO: replace all panic!s with JavaScript exception handling

pub fn apply(function: Value, this: Value, arguments: Vec<Value>, global: Object) -> Value {
    match &function {
        &Value::Function(ref f) => f.apply(this, arguments, global),
        _ => panic!("{:?} is not a function", function)
    }
}

fn eval_call(function: &ast::Expression, arguments: &ast::ExpressionList, local: Object, global: Object) -> Value {
    let func = eval_expression(function, local.clone(), global.clone());
    let args = eval_expression_list(arguments, local.clone(), global.clone());

    let this = match function {
        &ast::Expression::Access(ref a) => match a {
            &ast::Access::Member(_, _) => access_get(a, local, global.clone()),
            _ => Value::Object(global.clone())
        },
        _ => Value::Object(global.clone())
    };

    apply(func, this, args, global)
}

fn eval_expression_list(expressions: &Vec<ast::Expression>, local: Object, global: Object) -> Vec<Value> {
    let mut values = vec![];

    for e in expressions {
        let val: Value = eval_expression(e, local.clone(), global.clone());
        values.push(val);
    }

    values
}

fn access_get(access: &ast::Access, local: Object, global: Object) -> Value {
    match access {
        &ast::Access::Member(ref e, ref i) => eval_expression(e, local, global.clone()).get(i, global),
        &ast::Access::Identifier(ref i) => local.get(i)
    }
}

fn access_set(access: &ast::Access, local: Object, global: Object, val: Value) -> Value {
    match access {
        &ast::Access::Member(ref e, ref i) => eval_expression(e, local, global).set(i, val),
        &ast::Access::Identifier(ref i) => {
            local.outer_set(i, val);
            local.get(i)
        }
    }
}

fn eval_expression(expression: &ast::Expression, local: Object, global: Object) -> Value {
    match expression {
        &ast::Expression::Assignment(ref lhs, ref rhs) => {
            let rhs = eval_expression(rhs, local.clone(), global.clone());
            access_set(lhs, local, global, rhs)
        },
        &ast::Expression::Call(ref f, ref a) => eval_call(f, a, local, global),
        &ast::Expression::Access(ref a) => access_get(a, local, global),
        // TODO: get rid of clone
        &ast::Expression::Literal(ref l) => l.clone()
    }
}

fn eval_statement(statement: &ast::Statement, local: Object, global: Object) -> Value {
    match statement {
        &ast::Statement::Expression(ref e) => eval_expression(e, local, global).clone(),
        &ast::Statement::Declaration(ref d) => match d {
            &ast::Declaration::Variable(ref id, ref init) => {
                if let &Some(ref expr) = init {
                    local.set(id, eval_expression(expr, local.clone(), global.clone()));
                }

                Value::Undefined
            },
            _ => Value::Undefined
        },
        &ast::Statement::Empty => Value::Undefined
    }
}

pub fn eval(program: &ast::Block, local: Object, global: Object) -> Value {
    let mut last = Value::Undefined;

    // inefficient but convenient to parse
    for statement in program {
        if let &ast::Statement::Declaration(ref decl) = statement {
            match decl {
                &ast::Declaration::Variable(ref id, _) => local.set(id, Value::Undefined),
                &ast::Declaration::Function(ref id, ref f) => local.set(id, Value::Function(f.clone()))
            };
        }
    }

    for statement in program {
        last = eval_statement(statement, local.clone(), global.clone());
    }

    last
}
