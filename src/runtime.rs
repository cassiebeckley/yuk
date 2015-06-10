use super::parser;

use super::interpret;
use super::interpret::JSResult;

/// A high-level interface for the interpreter
pub struct Ack {
    global: interpret::Object
}

impl Ack {
    pub fn new() -> Ack {
        Ack {global: create_stdlib()}
    }

    pub fn eval(&mut self, source: &str) -> JSResult {
        let parsed = parser::parse(source);

        println!("");
        println!("AST: {:?}", parsed);

        match parsed {
            Ok(ast) => interpret::eval_block(&ast, self.global.clone(), self.global.clone()),
            Err(e) => interpret::throw_string(format!("SyntaxError: {:?}", e))
        }
    }
}

/// Create a default global object
pub fn create_stdlib() -> interpret::Object {
    use interpret::*;

    Object::from_map(hashmap!{
        "console".to_string() => Value::Object(Object::from_map(hashmap!{
            "log".to_string() => Value::Function(Function::Native(Rc::new(console_log)))
        })),
        "Number".to_string() => Value::Object(Object::from_map(hashmap!{
            "prototype".to_string() => Value::Object(Object::from_map(hashmap!{
                "toString".to_string() => Value::Function(Function::Native(Rc::new(number_prototype_to_string)))
            }))
        })),
        "String".to_string() => Value::Object(Object::from_map(hashmap!{
            "prototype".to_string() => Value::Object(Object::from_map(hashmap!{
                "toString".to_string() => Value::Function(Function::Native(Rc::new(string_prototype_to_string)))
            }))
        })),
    })
}

fn console_log(this: interpret::Value, arguments: Vec<interpret::Value>, _: interpret::Object) -> interpret::JSResult {
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
