use std::collections::HashMap;
use std::{fmt, iter, f64};

pub use std::rc::Rc;
pub use std::cell::RefCell;

use super::ast;

pub type JSResult = Result<Value, Value>;

trait GetFromResult {
    fn get(self, key: &str, global: Object) -> JSResult;
}

impl GetFromResult for JSResult {
    fn get(self, key: &str, global: Object) -> JSResult {
        match self {
            Ok(ref value) => value.get(key, global),
            Err(_) => self.clone()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Context {
    pub this: Value,
    pub local: Object,
    pub global: Object
}

#[derive(Debug, Clone)]
pub struct UserFunction {
    pub id: Option<ast::Identifier>,
    pub parameters: Vec<String>,
    pub body: ast::Block,
    pub source: String
}

pub enum Function {
    Native(fn(Vec<Value>, Context) -> JSResult),
    User(UserFunction),
}

impl fmt::Debug for Function {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Function::Native(_) => fmt.write_str("Native([native code])"),
            &Function::User (ref u) => u.fmt(fmt)
        }
    }
}

impl Function {
    fn apply(&self, arguments: Vec<Value>, context: Context) -> JSResult {
        match self {
            &Function::Native(ref f) => f(arguments, context),
            &Function::User (UserFunction {id: _, parameters: ref p, body: ref b, source: _}) => {
                let inner_env = Object::Object(Rc::new(RefCell::new(ActualObject::create(context.global.clone()))));
                let undef = Value::Undefined;
                for (argument, parameter) in arguments.iter().chain(iter::repeat(&undef)).zip(p) {
                    try!(inner_env.set(parameter, argument.clone()));
                }

                try!(eval_block(b, Context {this: context.this, local: inner_env.clone(), global: context.global}));

                Ok(Value::Undefined)
            }
        }
    }

    fn debug_string(&self) -> String {
        match self {
            &Function::Native(_) => "function() {\n    [native code]\n}".to_string(),
            &Function::User(UserFunction {id: Some(ref id), parameters: _, body: _, source: _}) => format!("function {}()", id),
            &Function::User(UserFunction {id: None, parameters: _, body: _, source: _}) => "function()".to_string(),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            &Function::Native(_) => "function() {\n    [native code]\n}".to_string(),
            &Function::User(UserFunction {id: _, parameters: _, body: _, source: ref s}) => s.to_string()
        }
    }
}

#[derive(Debug)]
pub enum ObjectExtension {
    Function(Function),
    None
}

#[derive(Debug)]
pub struct ActualObject {
    pub values: HashMap<String, Value>,
    pub prototype: Object,
    pub otype: ObjectExtension
}

impl ActualObject {
    pub fn new() -> ActualObject {
        ActualObject {
            values: HashMap::new(),
            prototype: Object::Null,
            otype: ObjectExtension::None
        }
    }

    pub fn create(proto: Object) -> ActualObject {
        let proto = proto.clone();

        ActualObject {
            values: HashMap::new(),
            prototype: proto,
            otype: ObjectExtension::None
        }
    }

    pub fn from_map(map: HashMap<String, Value>) -> ActualObject {
        ActualObject {
            values: map,
            prototype: Object::Null,
            otype: ObjectExtension::None
        }
    }

    fn outer_set(&mut self, key: &str, val: Value) -> Value {
        let key = key.to_string();
        if self.values.contains_key(&key) {
            self.values.insert(key, val.clone());
        } else if let Object::Object(ref proto) = self.prototype {
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
                Object::Object(ref proto) => proto.borrow().get(key),
                Object::Null => Value::Undefined
            }
        }
    }

    fn get_or_err(&self, key: &str) -> JSResult {
        match self.values.get(&key.to_string()) {
            Some(v) => Ok(v.clone()),
            None => match self.prototype {
                Object::Object(ref proto) => Ok(proto.borrow().get(key)),
                Object::Null => throw_string(format!("{} is not defined", key))
            }
        }
    }

    fn set(&mut self, key: &str, val: Value) -> Value {
        self.values.insert(key.to_string(), val.clone());
        val
    }

    fn debug_string(&self) -> String {
        match &self.otype {
            &ObjectExtension::Function(ref f) => f.debug_string(),
            &ObjectExtension::None => {
                let middle: String = self.values.iter()
                .map(|(key, value)| "\"".to_string() + key + "\": " + &value.debug_string())
                .fold("".to_string(), |result, next| if result.len() > 0 {result + ", " + &next} else {next});

                "{".to_string() + &middle + "}"
            }
        }
    }

    fn from_function(func: Function, prototype: Object) -> ActualObject {
        ActualObject {
            values: HashMap::new(),
            prototype: prototype,
            otype: ObjectExtension::Function(func)
        }
    }

    pub fn apply(&self, arguments: Vec<Value>, context: Context) -> JSResult {
        match &self.otype {
            &ObjectExtension::Function(ref f) => f.apply(arguments, context),
            _ => throw_string(format!("{:?} is not a function", self))
        }
    }
}

#[derive(Debug, Clone)]
pub enum Object {
    Object(Rc<RefCell<ActualObject>>),
    Null
}

impl Object {
    pub fn new() -> Object {
        Object::Object(Rc::new(RefCell::new(ActualObject::new())))
    }

    pub fn from_map(obj: HashMap<String, Value>) -> Object {
        Object::Object(Rc::new(RefCell::new(ActualObject::from_map(obj))))
    }

    pub fn from_function(func: Function, prototype: Object) -> Object {
        Object::Object(Rc::new(RefCell::new(ActualObject::from_function(func, prototype))))
    }

    fn outer_set(&self, key: &str, val: Value) -> JSResult {
        match self {
            &Object::Object(ref obj) => Ok(obj.borrow_mut().outer_set(key, val)),
            &Object::Null => throw_string("null has no properties".to_string())
        }
    }

    pub fn get(&self, key: &str) -> JSResult {
        match self {
            &Object::Object(ref obj) => Ok(obj.borrow().get(key)),
            &Object::Null => throw_string("null has no properties".to_string())
        }
    }

    pub fn get_or_err(&self, key: &str) -> JSResult {
        match self {
            &Object::Object(ref obj) => obj.borrow().get_or_err(key),
            &Object::Null => throw_string("null has no properties".to_string())
        }
    }

    pub fn set(&self, key: &str, val: Value) -> JSResult {
        match self {
            &Object::Object(ref obj) => Ok(obj.borrow_mut().set(key, val)),
            &Object::Null => throw_string("null has no properties".to_string())
        }
    }

    fn debug_string(&self) -> String {
        match self {
            &Object::Object(ref o) => o.borrow().debug_string(),
            &Object::Null => "null".to_string()
        }
    }
}

#[derive(Debug,Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Object(Object),
    Undefined
}

impl Value {
    pub fn get(&self, key: &str, global: Object) -> JSResult {
        match self {
            &Value::Number(_) => global.clone().get("Number").get("prototype", global.clone()).get(key, global),
            &Value::String(_) => global.clone().get("String").get("prototype", global.clone()).get(key, global),
            &Value::Object(ref obj) => obj.get(key),
            _ => throw_string(format!("{:?} is not an object", self))
        }
    }

    pub fn set(&self, key: &str, val: Value) -> JSResult {
        match self {
            &Value::Object(ref obj) => obj.set(key, val),
            _ => throw_string(format!("{:?} is not an object", self))
        }
    }

    pub fn outer_set(&self, key: &str, val: Value) -> JSResult {
        match self {
            &Value::Object(ref obj) => obj.outer_set(key, val),
            _ => throw_string(format!("{:?} is not an object", self))
        }
    }

    pub fn debug_string(&self) -> String {
        match self {
            &Value::Number(n) => n.to_string(),
            &Value::String(ref s) => s.to_string(),
            &Value::Object(ref o) => o.debug_string(),
            &Value::Undefined => "undefined".to_string(),
        }
    }

    pub fn from_function(func: Function, prototype: Object) -> Value {
        Value::Object(Object::from_function(func, prototype))
    }

    pub fn apply(&self, arguments: Vec<Value>, context: Context) -> JSResult {
        match self {
            &Value::Object(Object::Object(ref o)) => o.borrow().apply(arguments, context),
            _ => throw_string(format!("{:?} is not a function", self))
        }
    }

    // Conversions
    pub fn to_number(&self) -> f64 {
        match self {
            &Value::Number(n) => n,
            &Value::String(ref s) => s.parse().unwrap_or(f64::NAN),
            &Value::Object(_) => f64::NAN,
            &Value::Undefined => f64::NAN
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            &Value::Number(_) => String::new(),
            &Value::String(ref s) => s.clone(),
            &Value::Object(_) => String::new(),
            &Value::Undefined => String::new()
        }
    }
}

// TODO: Use proper exceptions, rather than strings
pub fn throw_string<T>(err: String) -> Result<T, Value> {
    Err(Value::String(err))
}

fn eval_call(function: &ast::Expression, arguments: &ast::ExpressionList, mut context: Context) -> JSResult {
    let func = try!(eval_expression(function, context.clone()));
    let args = try!(eval_expression_list(arguments, context.clone()));

    context.this = match function {
        &ast::Expression::Access(ref a) => match a {
            // TODO: This is probably bad -- p is evaluated **twice**, and therefore side effects happen twice
            &ast::Access::Member(ref p, _) => try!(eval_expression(p, context.clone())),
            _ => Value::Object(context.global.clone())
        },
        _ => Value::Object(context.global.clone())
    };

    func.apply(args, context)
}

fn eval_expression_list(expressions: &Vec<ast::Expression>, context: Context) -> Result<Vec<Value>, Value> {
    let mut values = vec![];

    for e in expressions {
        let val: Value = try!(eval_expression(e, context.clone()));
        values.push(val);
    }

    Ok(values)
}

fn access_get(access: &ast::Access, context: Context) -> JSResult {
    match access {
        &ast::Access::Member(ref e, ref i) => try!(eval_expression(e, context.clone())).get(i, context.global),
        &ast::Access::Identifier(ref i) => context.local.get_or_err(i)
    }
}

fn access_set(access: &ast::Access, context: Context, val: Value) -> JSResult {
    match access {
        &ast::Access::Member(ref e, ref i) => try!(eval_expression(e, context)).set(i, val),
        &ast::Access::Identifier(ref i) => context.local.outer_set(i, val)
    }
}

fn eval_unary(op: &ast::UnaryOp, exp: &ast::Expression, context: Context) -> JSResult {
    let val = try!(eval_expression(exp, context));

    match op {
        &ast::UnaryOp::Negative => Ok(Value::Number(-val.to_number())),
        &ast::UnaryOp::Positive => Ok(Value::Number(val.to_number()))
    }
}

fn eval_add(left: Value, right: Value, global: Object) -> JSResult {
    match (left.clone(), right.clone()) {
        (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left + right)),
        _ => {
            let left_string = try!(try!(left.get("toString", global.clone())
                .or(throw_string(format!("can't convert {} to primitive type", left.debug_string()))))
                .apply(vec![], Context {this: left.clone(), local: global.clone(), global: global.clone()})
                .or(throw_string(format!("can't convert {} to primitive type", left.debug_string())))).to_string();

            let right_string = try!(try!(right.get("toString", global.clone())
                .or(throw_string(format!("can't convert {} to primitive type", right.debug_string()))))
                .apply(vec![], Context {this: right.clone(), local: global.clone(), global: global.clone()})
                .or(throw_string(format!("can't convert {} to primitive type", right.debug_string())))).to_string();

            Ok(Value::String(left_string + &right_string))
        }
    }
}

fn eval_binary(op: &ast::BinaryOp, left: &ast::Expression, right: &ast::Expression, context: Context) -> JSResult {
    let left = try!(eval_expression(left, context.clone()));
    let right = try!(eval_expression(right, context.clone()));

    match op {
        &ast::BinaryOp::Add => eval_add(left, right, context.global),
        &ast::BinaryOp::Subtract => Ok(Value::Number(left.to_number() - right.to_number())),

        &ast::BinaryOp::Multiply => Ok(Value::Number(left.to_number() * right.to_number())),
        &ast::BinaryOp::Divide => Ok(Value::Number(left.to_number() / right.to_number()))
    }
}

fn eval_expression(expression: &ast::Expression, context: Context) -> JSResult {
    match expression {
        &ast::Expression::Assignment(ref lhs, ref rhs) => {
            let rhs = try!(eval_expression(rhs, context.clone()));
            access_set(lhs, context, rhs)
        },
        &ast::Expression::Call(ref f, ref a) => eval_call(f, a, context),
        &ast::Expression::Access(ref a) => access_get(a, context),
        &ast::Expression::Literal(ref l) => Ok(l.clone()),
        &ast::Expression::Function(ref uf) => {
            let fp = match try!(try!(context.global.get("Function")).get("prototype", context.global)) {
                Value::Object(o) => o,
                _ => try!(throw_string("Function.prototype must be an obect".to_string()))
            };
            Ok(Value::from_function(Function::User(uf.clone()), fp))
        },
        &ast::Expression::Unary(ref u, ref e) => eval_unary(u, e, context),
        &ast::Expression::Binary(ref b, ref l, ref r) => eval_binary(b, l, r, context),
        &ast::Expression::This => Ok(context.this)
    }
}

fn eval_statement(statement: &ast::Statement, context: Context) -> JSResult {
    match statement {
        &ast::Statement::Expression(ref e) => eval_expression(e, context),
        &ast::Statement::Declaration(ref d) => match d {
            &ast::Declaration::Variable(ref id, ref init) => {
                if let &Some(ref expr) = init {
                    try!(context.local.set(id, try!(eval_expression(expr, context.clone()))));
                }

                Ok(Value::Undefined)
            },
            _ => Ok(Value::Undefined)
        },
        &ast::Statement::Throw(ref e) => Err(try!(eval_expression(e, context))),
        &ast::Statement::Empty => Ok(Value::Undefined)
    }
}

pub fn eval_block(program: &ast::Block, context: Context) -> JSResult {
    let mut last = Value::Undefined;

    let function_prototype = match try!(context.global.get("Function").get("prototype", context.global.clone())) {
        Value::Object(o) => o,
        _ => return throw_string("Function.prototype must be an object".to_string())
    };

    // inefficient (I think) but convenient to parse
    for statement in program {
        if let &ast::Statement::Declaration(ref decl) = statement {
            match decl {
                &ast::Declaration::Variable(ref id, _) => try!(context.local.set(id, Value::Undefined)),
                &ast::Declaration::Function(ref id, ref f) => try!(context.local.set(id, Value::from_function(Function::User(f.clone()), function_prototype.clone())))
            };
        }
    }

    for statement in program {
        last = try!(eval_statement(statement, context.clone()));
    }

    Ok(last)
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_eval_add() {
//         let cases = vec![
//             (Value::Number(12.4), Value::Number(27.6), Value::Number(40)),
//             (Value::String("hello".to_string()), Value::Number(5), Value::String("hello5".to_string())),
//             (Value::Number(27), Value::String("yup"), Value::String("27yup".to_string())),
//         ];

//         for (left, right, total) in cases {
//             assert_eq!(total, eval_add(left, right, Object::new()))
//         }
//     }
// }
