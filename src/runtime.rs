use super::parser;

use super::interpret;
use super::interpret::{JSResult, Context};

use std::ops::Deref;

macro_rules! object {
    ( $($key:ident => $value:expr),* ) => {
        Object::from_map(hashmap! {
            $(stringify!($key).to_string() => $value.to_value()),*
        })
    }
}

/// Create a native function
macro_rules! function {
    ( $f:ident ($context:ident ; $( $x:ident ),* ; $args:ident ) $body:block , $prototype:expr ) => {
        {
            fn native(arguments: Vec<interpret::Value>, mut $context: interpret::Context) -> interpret::JSResult {
                let mut arguments = arguments.into_iter();
                $(
                    let $x = arguments.next().unwrap_or(Value::Undefined);
                )*
                let $args = arguments.collect();
                $body
            }
            Value::from_function(Function::Native(stringify!($f).to_string(), native), $prototype)
        }
    };
    ( $f:ident ($context:ident ; $( $t:ident $x:ident ),* ; $args:ident ) $body:block , $prototype:expr ) => {
        {
            fn native(arguments: Vec<interpret::Value>, mut $context: interpret::Context) -> interpret::JSResult {
                let mut arguments = arguments.into_iter();
                $(
                    let $x = match arguments.next().unwrap_or(Value::Undefined) {
                        Value::$t(b) => b,
                        o@_ => return interpret::throw_string(format!("{:?} is not a {}", o, stringify!($t)))
                    };
                )*
                let $args: Vec<interpret::Value> = arguments.collect();
                $body
            }
            Value::from_function(Function::Native(stringify!($f).to_string(), native), $prototype)
        }
    };
    ( $f:ident ($context:ident ; $args:ident ) $body:block , $prototype:expr ) => {
        {
            fn native(arguments: Vec<interpret::Value>, $context: interpret::Context) -> interpret::JSResult {
                let $args = arguments;
                $body
            }
            Value::from_function(Function::Native(stringify!($f).to_string(), native), $prototype)
        }
    };
}

pub type Ack = interpret::Context;

/// A high-level interface for the interpreter
impl Ack {
    /// Create a context with the JavaScript standard library
    pub fn create_stdlib() -> Ack {
        Context::new(create_stdlib())
    }

    /// Parse and evaluate `source` in this context
    pub fn eval(&mut self, source: &str) -> JSResult {
        let parsed = parser::parse(source);

        // println!("");
        // println!("AST: {:?}", parsed);

        match parsed {
            Ok(ast) => match interpret::eval_block(&ast, self.clone()) {
                interpret::Tri::Continue(v) => Ok(v),
                interpret::Tri::Return(v) => Ok(v),
                interpret::Tri::Error(e) => Err(e)
            },
            Err(e) => interpret::throw_string(format!("SyntaxError: {:?}", e))
        }
    }
}

/// This is private, anyway
fn create_stdlib() -> interpret::Object {
    use interpret::*;

    let function_prototype = Object::new();

    function_prototype.set("toString", function!(
        toString(context; _args) {
            match context.this {
                interpret::Value::Object(interpret::Object::Object(ref o)) => match o.borrow().deref() {
                    &interpret::ActualObject {values: _, prototype: _, otype: interpret::ObjectExtension::Function(ref f)} => Ok(interpret::Value::String(f.to_string())),
                    _ => interpret::throw_string(format!("{:?} is not a function!", context.this))
                },
                _ => interpret::throw_string(format!("{:?} is not a function!", context.this))
            }
        }, function_prototype.clone()
    )).unwrap();

    object! {
        console => object! {
            log => function!(
                log(_context; arguments) {
                    for value in arguments {
                        print!("{} ", value.debug_string());
                    }
                    println!("");

                    Ok(interpret::Value::Undefined)
                }, function_prototype.clone()
            )
        },
        Number => object! {
            prototype => object! {
                toString => function! (
                    toString(context; _args) {
                        match context.this {
                            interpret::Value::Number(n) => Ok(interpret::Value::String(n.to_string())),
                            _ => interpret::throw_string(format!("{:?} is not a number!", context.this))
                        }
                    }, function_prototype.clone()
                )
            }
        },
        String => object! {
            prototype => object! {
                toString => function! (
                    toString(context; _args) {
                        match context.this {
                            interpret::Value::String(s) => Ok(interpret::Value::String(s.to_string())),
                            _ => interpret::throw_string(format!("{:?} is not a string!", context.this))
                        }
                    }, function_prototype.clone()
                )
            }
        },
        eval => function!(
            eval(context; String source; _args) {
                context.eval(&source)
            }, function_prototype.clone()
        ),
        Function => object! {
            prototype => function_prototype
        }
    }
}
