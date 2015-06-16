use std::collections::HashMap;
use std::{fmt, iter, f64, cmp};
use std::ops::Deref;

pub use std::rc::Rc;
pub use std::cell::RefCell;

use super::ast;

pub type JSResult = Result<Value, Value>;

pub trait ToValue {
    fn to_value(&self) -> Value;
}

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

/// Contains the state for an interpreter thread
#[derive(Debug, Clone)]
pub struct Context {
    pub this: Value,
    pub local: Object,
    pub global: Object
}

impl Context {
    /// Creates a top-level context
    ///
    /// `this`, `local`, and `global` are all set to `obj`.
    pub fn new(obj: Object) -> Context {
        Context {
            this: Value::Object(obj.clone()),
            local: obj.clone(),
            global: obj
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserFunction {
    pub function: ast::Function,
    pub local: Object
}

impl UserFunction {
    // Creates a UserFunction from `function` and `local`
    pub fn new(function: ast::Function, local: Object) -> UserFunction {
        UserFunction {
            function: function,
            local: local
        }
    }
}

pub enum Function {
    Native(String, fn(Vec<Value>, Context) -> JSResult),
    User(UserFunction),
}

impl fmt::Debug for Function {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Function::Native(ref id, _) => fmt.write_str(&format!("Native({}, [native code])", id)),
            &Function::User (ref u) => u.fmt(fmt)
        }
    }
}

impl Function {
    fn apply(&self, arguments: Vec<Value>, context: Context) -> JSResult {
        match self {
            &Function::Native(_, ref f) => f(arguments, context),
            &Function::User (UserFunction {function: ast::Function {id: _, parameters: ref p, body: ref b, source: _}, local: ref closure_scope}) => {
                let inner_env = Object::Object(Rc::new(RefCell::new(ActualObject::create(closure_scope.clone()))));
                let undef = Value::Undefined;
                for (argument, parameter) in arguments.iter().chain(iter::repeat(&undef)).zip(p) {
                    try!(inner_env.set(parameter, argument.clone()));
                }

                match eval_inner_block(b, Context {this: context.this, local: inner_env.clone(), global: context.global}) {
                    Tri::Continue(v) => Ok(v),
                    Tri::Return(v) => Ok(v),
                    Tri::Error(v) => Err(v)
                }
            }
        }
    }

    fn debug_string(&self) -> String {
        match self {
            &Function::Native(ref id, _) => format!("function {}()", id),
            &Function::User(UserFunction {function: ast::Function {id: Some(ref id), parameters: _, body: _, source: _}, local: _ }) => format!("function {}()", id),
            &Function::User(UserFunction {function: ast::Function {id: None, parameters: _, body: _, source: _}, local: _}) => "function()".to_string(),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            &Function::Native(ref id, _) => format!("function {}() {{\n    [native code]\n}}", id),
            &Function::User(UserFunction {function: ast::Function {id: _, parameters: _, body: _, source: ref s}, local: _}) => s.to_string()
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

impl ToValue for Object {
    fn to_value(&self) -> Value {
        Value::Object(self.clone())
    }
}

impl cmp::PartialEq for Object {
    fn eq(&self, other: &Object) -> bool {
        match (self, other) {
            (&Object::Object(ref a_rc), &Object::Object(ref b_rc)) => {
                let a: *const RefCell<ActualObject> = a_rc.deref();
                let b: *const RefCell<ActualObject> = b_rc.deref();
                a == b
            },
            (&Object::Null, &Object::Null) => true,
            _ => false
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    String(String),
    Object(Object),
    Undefined
}

impl Value {
    pub fn get(&self, key: &str, global: Object) -> JSResult {
        match self {
            &Value::Number(_) => global.clone().get("Number").get("prototype", global.clone()).get(key, global),
            &Value::Boolean(_) => global.clone().get("Boolean").get("prototype", global.clone()).get(key, global),
            &Value::String(_) => global.clone().get("String").get("prototype", global.clone()).get(key, global),
            &Value::Object(ref obj) => obj.get(key),
            &Value::Undefined => throw_string("undefined has no properties".to_string())
        }
    }

    pub fn set(&self, key: &str, val: Value) -> JSResult {
        match self {
            &Value::Number(_) => Ok(Value::Undefined),
            &Value::Boolean(_) => Ok(Value::Undefined),
            &Value::String(_) => Ok(Value::Undefined),
            &Value::Object(ref obj) => obj.set(key, val),
            &Value::Undefined => throw_string("undefined has no properties".to_string())
        }
    }

    pub fn outer_set(&self, key: &str, val: Value) -> JSResult {
        match self {
            &Value::Object(ref obj) => obj.outer_set(key, val),
            _ => throw_string(format!("{:?} is not an object", self))
        }
    }

    /// Formats output for debugging functions
    pub fn debug_string(&self) -> String {
        match self {
            &Value::Number(n) => n.to_string(),
            &Value::Boolean(b) => b.to_string(),
            &Value::String(ref s) => s.to_string(),
            &Value::Object(ref o) => o.debug_string(),
            &Value::Undefined => "undefined".to_string(),
        }
    }

    // TODO: figure out better naming conventions
    /// Converts value to string using its `toString` attribute
    pub fn js_to_string(&self, global: Object) -> Result<String, Value> {
        self.get("toString", global.clone())
            .and_then(|to_string| to_string.apply(vec![], Context {this: self.clone(), local: global.clone(), global: global.clone()}))
            .map(|val| val.to_string())
            .or(throw_string(format!("can't convert {} to primitive type", self.debug_string())))
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

    fn add(&self, right: &Value, global: Object) -> JSResult {
        match (self.clone(), right.clone()) {
            (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left + right)),

            (left, Value::String(right)) => Ok(Value::String(try!(left.js_to_string(global)) + &right)),
            (Value::String(left), right) => {
                let right = try!(right.js_to_string(global));
                Ok(Value::String(left + &right))
            },
            _ => {
                let (left_num, right_num) = (self.to_number(), right.to_number());

                if !left_num.is_nan() && !right_num.is_nan() {
                    Ok(Value::Number(left_num + right_num))
                } else {
                    let left_string = try!(self.js_to_string(global.clone()));
                    let right_string = try!(right.js_to_string(global));

                    Ok(Value::String(left_string + &right_string))
                }
            }
        }
    }

    pub fn strict_equals(&self, right: &Value) -> bool {
        self == right
    }

    // Conversions
    pub fn to_number(&self) -> f64 {
        match self {
            &Value::Number(n) => n,
            &Value::Boolean(b) => if b {1.0} else {0.0},
            &Value::String(ref s) => s.parse().unwrap_or(f64::NAN),
            &Value::Object(_) => f64::NAN,
            &Value::Undefined => f64::NAN
        }
    }

    pub fn to_boolean(&self) -> bool {
        match self {
            &Value::Number(n) => n != 0.0,
            &Value::Boolean(b) => b,
            &Value::String(ref s) => !s.is_empty(),
            &Value::Object(_) => true,
            &Value::Undefined => false
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            &Value::Number(_) => String::new(),
            &Value::Boolean(_) => String::new(),
            &Value::String(ref s) => s.clone(),
            &Value::Object(_) => String::new(),
            &Value::Undefined => String::new()
        }
    }
}

impl ToValue for Value {
    fn to_value(&self) -> Value {
        self.clone()
    }
}

// TODO: Use proper exceptions, rather than strings
pub fn throw_string<T>(err: String) -> Result<T, Value> {
    Err(Value::String(err))
}

// TODO: come up with better name, or get rid of this altogether
// make it work nicely at the least
#[must_use]
#[derive(Debug)]
pub enum Tri {
    Continue(Value),
    Return(Value),
    Error(Value)
}

fn eval_inner_block(block: &ast::InnerBlock, context: Context) -> Tri {
    match eval_block(&block.block, context.clone()) {
        Tri::Continue(_) => (),
        Tri::Return(v) => return Tri::Return(v),
        Tri::Error(e) => return Tri::Error(e)
    }

    match &block.return_exp {
        &Some(ref e) => match eval_expression(e, context) {
            Ok(v) => Tri::Return(v),
            Err(e) => Tri::Error(e)
        },
        &None => Tri::Continue(Value::Undefined)
    }
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

fn eval_accessor(acor: &ast::Accessor, context: Context) -> Result<String, Value> {
    match acor {
        &ast::Accessor::Identifier(ref id) => Ok(id.clone()),
        &ast::Accessor::Expression(ref e) => try!(eval_expression(e, context.clone())).js_to_string(context.global)
    }
}

fn access_get(access: &ast::Access, context: Context) -> JSResult {
    match access {
        &ast::Access::Member(ref e, ref a) => {
            let id = try!(eval_accessor(a, context.clone()));
            eval_expression(e, context.clone()).get(&id, context.global)
        },
        &ast::Access::Identifier(ref i) => context.local.get_or_err(i)
    }
}

fn access_set(access: &ast::Access, context: Context, val: Value) -> JSResult {
    match access {
        &ast::Access::Member(ref e, ref a) => {
            let id = try!(eval_accessor(a, context.clone()));
            try!(eval_expression(e, context)).set(&id, val)
        },
        &ast::Access::Identifier(ref i) => context.local.outer_set(i, val)
    }
}

fn eval_unary(op: &ast::UnaryOp, exp: &ast::Expression, context: Context) -> JSResult {
    let val = try!(eval_expression(exp, context));

    match op {
        &ast::UnaryOp::Positive => Ok(Value::Number(val.to_number())),
        &ast::UnaryOp::Negative => Ok(Value::Number(-val.to_number())),

        &ast::UnaryOp::LogicalNot => Ok(Value::Boolean(!val.to_boolean())),
    }
}

fn eval_binary(op: &ast::BinaryOp, left: &ast::Expression, right: &ast::Expression, context: Context) -> JSResult {
    let left = try!(eval_expression(left, context.clone()));

    match op {
        &ast::BinaryOp::Add => left.add(&try!(eval_expression(right, context.clone())), context.global),
        &ast::BinaryOp::Subtract => Ok(Value::Number(left.to_number() - try!(eval_expression(right, context.clone())).to_number())),
        &ast::BinaryOp::LogicalAnd => Ok(if left.to_boolean() {
            let right = try!(eval_expression(right, context.clone()));
            if right.to_boolean() {
                right
            } else {
                Value::Boolean(false)
            }
        } else {
            Value::Boolean(false)
        }),
        &ast::BinaryOp::LogicalOr => Ok(if left.to_boolean() {
            left
        } else {
            let right = try!(eval_expression(right, context.clone()));
            if right.to_boolean() {
                right
            } else {
                Value::Boolean(false)
            }
        }),

        &ast::BinaryOp::Multiply => Ok(Value::Number(left.to_number() * try!(eval_expression(right, context.clone())).to_number())),
        &ast::BinaryOp::Divide => Ok(Value::Number(left.to_number() / try!(eval_expression(right, context.clone())).to_number()))
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
        &ast::Expression::Function(ref f) => {
            let fp = match try!(try!(context.global.get("Function")).get("prototype", context.global)) {
                Value::Object(o) => o,
                _ => try!(throw_string("Function.prototype must be an object".to_string()))
            };
            Ok(Value::from_function(Function::User(UserFunction::new(f.clone(), context.local)), fp))
        },
        &ast::Expression::Unary(ref u, ref e) => eval_unary(u, e, context),
        &ast::Expression::Binary(ref b, ref l, ref r) => eval_binary(b, l, r, context),
        &ast::Expression::Ternary(ref condition, ref consequent, ref alternate) => {
            if try!(eval_expression(condition, context.clone())).to_boolean() {
                eval_expression(consequent, context)
            } else {
                eval_expression(alternate, context)
            }
        },
        &ast::Expression::Object(ref exprs) => {
            let mut obj = HashMap::new();
            for &(ref key, ref expr) in exprs {
                obj.insert(key.clone(), try!(eval_expression(expr, context.clone())));
            }

            Ok(Value::Object(Object::from_map(obj)))
        },
        &ast::Expression::This => Ok(context.this)
    }
}

fn eval_statement(statement: &ast::Statement, context: Context) -> Tri {
    match statement {
        &ast::Statement::Expression(ref e) => match eval_expression(e, context) {
            Ok(v) => Tri::Continue(v),
            Err(e) => Tri::Error(e)
        },
        &ast::Statement::Declaration(ref d) => match d {
            &ast::Declaration::Variable(ref id, ref init) => {
                if let &Some(ref expr) = init {
                    let init_val = match eval_expression(expr, context.clone()) {
                        Ok(v) => v,
                        Err(e) => return Tri::Error(e)
                    };

                    match context.local.set(id, init_val) {
                        Ok(_) => (),
                        Err(e) => return Tri::Error(e)
                    };
                }

                Tri::Continue(Value::Undefined)
            },
            _ => Tri::Continue(Value::Undefined)
        },
        &ast::Statement::Throw(ref e) => {
            let error_val = match eval_expression(e, context) {
                Ok(v) => v,
                Err(e) => return Tri::Error(e)
            };

            Tri::Error(error_val)
        },
        &ast::Statement::If(ref condition, ref consequent, ref alternate) => {
            let condition = match eval_expression(condition, context.clone()) {
                Ok(v) => v,
                Err(e) => return Tri::Error(e)
            };

            if condition.to_boolean() {
                eval_inner_block(consequent, context)
            } else if let &Some(ref alt) = alternate {
                eval_inner_block(alt, context)
            } else {
                Tri::Continue(Value::Undefined)
            }
        }
        &ast::Statement::Empty => Tri::Continue(Value::Undefined)
    }
}

pub fn eval_block(program: &ast::Block, context: Context) -> Tri {
    let mut last = Value::Undefined;

    let function_prototype = match context.global.get("Function").get("prototype", context.global.clone()) {
        Ok(Value::Object(o)) => o,
        Err(e) => return Tri::Error(e),
        _ => return Tri::Error(Value::String("Function.prototype must be an object".to_string()))
    };

    // inefficient (I think) but convenient to parse
    for statement in program {
        if let &ast::Statement::Declaration(ref decl) = statement {
            match decl {
                &ast::Declaration::Variable(ref id, _) => match context.local.set(id, Value::Undefined) {
                    Ok(_) => (),
                    Err(e) => return Tri::Error(e)
                },
                &ast::Declaration::Function(ref id, ref f) => {
                    let function = Value::from_function(Function::User(UserFunction::new(f.clone(), context.local.clone())), function_prototype.clone());
                    match context.local.set(id, function) {
                        Ok(_) => (),
                        Err(e) => return Tri::Error(e)
                    }
                }
            };
        }
    }

    for statement in program {
        last = match eval_statement(statement, context.clone()) {
            Tri::Continue(v) => v,
            Tri::Return(v) => return Tri::Return(v),
            Tri::Error(e) => return Tri::Error(e)
        };
    }

    Tri::Continue(last)
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
