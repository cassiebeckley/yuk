use super::parser;

use super::interpret;
use super::interpret::JSResult;

use std::ops::Deref;

pub type Ack = interpret::Context;

/// A high-level interface for the interpreter
impl Ack {
    pub fn new() -> Ack {
        let obj = create_stdlib();
        Ack {
            this: interpret::Value::Object(obj.clone()),
            local: obj.clone(),
            global: obj.clone()
        }
    }

    pub fn eval(&mut self, source: &str) -> JSResult {
        let parsed = parser::parse(source);

        // println!("");
        // println!("AST: {:?}", parsed);

        match parsed {
            Ok(ast) => interpret::eval_block(&ast, self.clone()),
            Err(e) => interpret::throw_string(format!("SyntaxError: {:?}", e))
        }
    }
}

/// Create a default global object
pub fn create_stdlib() -> interpret::Object {
    use interpret::*;

    let function_prototype = Object::new();
    function_prototype.set("toString", Value::from_function(Function::Native(function_prototype_to_string), function_prototype.clone())).unwrap();

    Object::from_map(hashmap!{
        "console".to_string() => Value::Object(Object::from_map(hashmap!{
            "log".to_string() => Value::from_function(Function::Native(console_log), function_prototype.clone())
        })),
        "Number".to_string() => Value::Object(Object::from_map(hashmap!{
            "prototype".to_string() => Value::Object(Object::from_map(hashmap!{
                "toString".to_string() => Value::from_function(Function::Native(number_prototype_to_string), function_prototype.clone())
            }))
        })),
        "String".to_string() => Value::Object(Object::from_map(hashmap!{
            "prototype".to_string() => Value::Object(Object::from_map(hashmap!{
                "toString".to_string() => Value::from_function(Function::Native(string_prototype_to_string), function_prototype.clone())
            }))
        })),
        "Function".to_string() => Value::Object(Object::from_map(hashmap!{
            "prototype".to_string() => Value::Object(function_prototype)
        })),
    })
}

fn console_log(_: interpret::Value, arguments: Vec<interpret::Value>, _: interpret::Object) -> interpret::JSResult {
    for value in arguments {
        print!("{} ", value.debug_string());
    }
    println!("");

    Ok(interpret::Value::Undefined)
}

fn number_prototype_to_string(this: interpret::Value, _: Vec<interpret::Value>, _: interpret::Object) -> interpret::JSResult {
    match this {
        interpret::Value::Number(n) => Ok(interpret::Value::String(n.to_string())),
        _ => interpret::throw_string(format!("{:?} is not a number!", this))
    }
}

fn string_prototype_to_string(this: interpret::Value, _: Vec<interpret::Value>, _: interpret::Object) -> interpret::JSResult {
    match this {
        interpret::Value::String(s) => Ok(interpret::Value::String(s.to_string())),
        _ => interpret::throw_string(format!("{:?} is not a string!", this))
    }
}

fn function_prototype_to_string(this: interpret::Value, _: Vec<interpret::Value>, _: interpret::Object) -> interpret::JSResult {
    match this {
        interpret::Value::Object(interpret::Object::Object(ref o)) => match o.borrow().deref() {
            &interpret::ActualObject {values: _, prototype: _, otype: interpret::ObjectExtension::Function(ref f)} => Ok(interpret::Value::String(f.to_string())),
            _ => interpret::throw_string(format!("{:?} is not a function!", this))
        },
        _ => interpret::throw_string(format!("{:?} is not a function!", this))
    }
}
