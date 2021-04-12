#![feature(or_patterns)]

mod arithparser;

use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use std::sync::Mutex;
use std::fmt;
use regex::Regex;

use lazy_static::lazy_static;

lazy_static! {
    static ref ENV: Mutex<Environment> = Mutex::new(Environment::new(vec![]));
}

#[wasm_bindgen]
extern {
    pub fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    pub fn error(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    pub fn warn(s: &str);
    #[wasm_bindgen]
    pub fn eval(s: &str) -> f64;
}

#[wasm_bindgen]
pub fn produce(name: &str) -> String {
    log("hello, world!");
    format!("hello, {}", name)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Private
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Pushn(f64),
    Pushs(String),
    Define(Visibility),
    EndDefine,
    Jmp,
    JmpIf,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    GT,
    GE,
    LT,
    LE,
    EQ,
    NE,
    Reference(String, usize),
    Print,
    Not,
    Dup,
    Swap,
    Drop,
    Put,
    Get,
    ArrowPut,
    AddressOf,
    SubProg,
    Lambda,
    End,
    Run,
    Return,
    Pull,

    Nop,
    Include,
    PrintStack,
    JSEval,
    Placeholder
}

#[derive(Debug, Clone)]
pub enum StackSlot {
    Number(f64),
    String(String),
    Reference(String, usize),
    Code(Vec<Command>)
}

#[derive(Clone, Debug)]
struct Stack {
    stack: Vec<StackSlot>
}

#[derive(Debug)]
struct RuntimeError {
    msg: String,
    call_stack: Vec<(usize, String)>
}

#[derive(Clone, Debug)]
struct Environment {
    prefix: Vec<String>,
    stack: Stack,
    definitions: HashMap<String, usize>,
    program: Vec<Command>,
    lambda_counter: usize,
    idx: usize
}

impl Environment {
    fn new(program: Vec<Command>) -> Self {
        Environment {
            prefix: Vec::new(),
            stack: Stack { stack: Vec::new() },
            definitions: HashMap::new(),
            program: program,
            lambda_counter: 0,
            idx: 0
        }
    }

    fn from(from: Environment) -> Self {
        Environment {
            prefix: from.prefix,
            stack: from.stack,
            definitions: from.definitions,
            program: from.program,
            lambda_counter: from.lambda_counter,
            idx: from.idx
        }
    }

    fn resolve_reference(definitions: &HashMap<String, usize>, name: String) -> Result<usize, String> {
        if definitions.contains_key(&name) {
            Ok(definitions[&name])
        } else {
            Err(format!("no such symbol: `{}`", name))
        }
    }

    fn define_new(&mut self, s: String) {
        self.prefix.push(s.clone());
        let name = self.prefix.join("::");

        if self.definitions.contains_key(&name) {
            *self.definitions.get_mut(&name).unwrap() = self.idx;
        } else {
            self.definitions.insert(
                name, self.idx
            );
        }
    }
}

impl Stack {
    fn push(&mut self, item: StackSlot) {
        self.stack.push(item);
    }

    fn pop(&mut self) -> Option<StackSlot> {
        self.stack.pop()
    }
}

impl RuntimeError {
    fn new(msg: String, call_stack: Vec<(usize, String)>) -> Self {
        RuntimeError {
            msg, call_stack
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let call_info = self.call_stack.iter().map(|f| f.1.clone()).collect::<Vec<String>>();
        write!(f, "RuntimeError: {}\ncallstack: {:#?}", self.msg, call_info)
    }
}

impl fmt::Display for Stack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.stack.iter().map(|e| format!("{:?}<br>", e)).fold(String::default(), |acc, e| acc + &e))
    }
}

fn run(env: &mut Environment) -> Result<Environment, RuntimeError> {
    let mut execute = true;
    let mut level = 0;

    let mut call_stack: Vec<(usize, String)> = Vec::new();

    while env.idx < env.program.len() { 
        match &env.program[env.idx] {
            Command::Define(v) => {
                if let Visibility::Public = v {
                    if let Command::Pushs(string) = env.program[env.idx - 1].clone() {
                        env.define_new(string.to_string());
                        execute = false;
                        level += 1;
                    }
                    else {
                        return Err(RuntimeError::new("public define needs a label".into(), call_stack));
                    }
                } else if execute {
                    if let Some(StackSlot::String(string)) = env.stack.pop() {
                        env.define_new(string.to_string());
                        execute = false;
                        level += 1;
                    }
                    else {
                        return Err(RuntimeError::new("string required for private define".into(), call_stack));
                    }
                }
            },
            Command::Lambda => {
                level += 1;
            },
            _ => {}
        }

        if !execute {
            match &env.program[env.idx] {
                Command::Lambda => {
                    env.define_new("lambda".into());
                },
                Command::EndDefine => {
                    env.prefix.pop();
                    level -= 1;
                    if level < 1 {
                        env.stack.pop();

                        execute = true;
                        env.idx += 1;
                        continue;
                    }
                },
                _ => {}
            }
        }

        if execute {
            match &env.program[env.idx] {
                Command::Nop => { },
                Command::Include => {
                    
//                    if let StackSlot::String(filename) = env.stack.pop().unwrap() {
                        //let comment = Regex::new(r"/\*.*\*/").unwrap();
/*
                        let mut file = File::open(&filename)
                            .expect("include: file not found");

                        let mut content = String::new();

                        file.read_to_string(&mut content)
                            .expect("unable to read file");

                        let result = comment.replace_all(content.as_ref(), "");
                        let mut tokens = lexer(result.to_string()).env.program;
                        tokens.reverse();

                        for token in tokens.iter() {
                            env.program.insert(env.idx + 1, token.clone());
                        }
                    } else {
                        return Err(RuntimeError::new("expected file name for include");
                    }
                    */
                }
                Command::Pushn(n) => env.stack.push(StackSlot::Number(*n)),
                Command::Pushs(s) => {
                    env.stack.push(StackSlot::String(s.clone()))
                },
                Command::EndDefine | Command::Return => if call_stack.len() > 0 {
                    env.idx = call_stack.pop().unwrap().0 as usize;
                },
                Command::JmpIf => {
                    if let Some(StackSlot::Reference(n, offset)) = env.stack.pop() {
                        if let Some(StackSlot::Number(f)) = env.stack.pop() {
                            if f != 0.0 {
                                call_stack.push((env.idx, n.clone()));
                                env.idx = env.definitions[&n] + offset;
                            }
                        } else {
                            return Err(RuntimeError::new("expected number for a conditional jump".into(), call_stack))

                        }
                    } else {
                        return Err(RuntimeError::new("expected reference for a jump".into(), call_stack));

                    }
                }
                Command::Jmp => {
                    if let StackSlot::Reference(n, offset) = env.stack.pop().unwrap() {
                        call_stack.push((env.idx, n.clone()));
                        if env.definitions.contains_key(&n) {
                            env.idx = env.definitions[&n] + offset;
                        } else {
                            return Err(RuntimeError::new("reference not found in definitions".into(), call_stack));
                        }
                    } else {
                        return Err(RuntimeError::new("expected reference for a jump".into(), call_stack));
                    }
                }
                Command::Add => {
                    if let (Some(right), Some(left)) = (env.stack.pop(), env.stack.pop()) {
                        if let (StackSlot::Number(r), StackSlot::Number(l)) = (&right, &left) {
                            env.stack.push(StackSlot::Number(l + r));
                        } else if let (StackSlot::String(r), StackSlot::Number(l)) = (&right, &left) {
                            env.stack.push(StackSlot::String(
                                format!("{}{}", l, r)
                            ));
                        } else if let (StackSlot::Number(r), StackSlot::String(l)) = (&right, &left) {
                            env.stack.push(StackSlot::String(
                                format!("{}{}", l, r)
                            ));
                        } else if let (StackSlot::String(r), StackSlot::String(l)) = (&right, &left) {
                            env.stack.push(StackSlot::String(
                                format!("{}{}", l, r)
                            ));
                        } else if let (StackSlot::Number(r), StackSlot::Reference(name, l)) = (&right, &left) {
                            env.stack.push(StackSlot::Reference(
                                name.clone(), *r as usize + l
                            ));
                        } else {
                            return Err(RuntimeError::new("add operator only supported for numbers or strings".into(), call_stack));
                        }
                    } else {
                        return Err(RuntimeError::new("stack underflow while adding!".into(), call_stack));
                    }
                },
                Command::Sub => {
                    if let (Some(right), Some(left)) = (env.stack.pop(), env.stack.pop()) {
                        if let (StackSlot::Number(r), StackSlot::Number(l)) = (right, left) {
                            env.stack.push(StackSlot::Number(l - r));
                        } else {
                            return Err(RuntimeError::new("arithmetic is only supported for numbers".into(), call_stack));
                        }
                    } else {
                        return Err(RuntimeError::new("stack underflow while subtracting!".into(), call_stack));
                    }

                },
                Command::Mul => {
                    if let (Some(right), Some(left)) = (env.stack.pop(), env.stack.pop()) {
                        if let (StackSlot::Number(r), StackSlot::Number(l)) = (right, left) {
                            env.stack.push(StackSlot::Number(l * r));
                        } else {
                            return Err(RuntimeError::new("arithmetic is only supported for numbers".into(), call_stack));
                        }
                    } else {
                        return Err(RuntimeError::new("stack underflow while multiplying!".into(), call_stack));
                    }
                },
                Command::Div => {
                    if let (Some(right), Some(left)) = (env.stack.pop(), env.stack.pop()) {
                        if let (StackSlot::Number(r), StackSlot::Number(l)) = (right, left) {
                            env.stack.push(StackSlot::Number(l / r));
                        } else {
                            return Err(RuntimeError::new("arithmetic is only supported for numbers".into(), call_stack));
                        }
                    } else {
                        return Err(RuntimeError::new("stack underflow while dividing!".into(), call_stack));
                    }
                },
                Command::Mod => {
                    if let (Some(right), Some(left)) = (env.stack.pop(), env.stack.pop()) {
                        if let (StackSlot::Number(r), StackSlot::Number(l)) = (right, left) {
                            env.stack.push(StackSlot::Number(l % r));
                        } else {
                            return Err(RuntimeError::new("arithmetic is only supported for numbers".into(), call_stack));
                        }
                    } else {
                        return Err(RuntimeError::new("stack underflow in modulo operation!".into(), call_stack));
                    }
                },
                Command::LT => {
                    if let (Some(right), Some(left)) = (env.stack.pop(), env.stack.pop()) {
                        if let (StackSlot::Number(r), StackSlot::Number(l)) = (right, left) {
                            env.stack.push(StackSlot::Number(if l < r { 1.0 } else { 0.0 }));
                        } else {
                            return Err(RuntimeError::new("arithmetic is only supported for numbers".into(), call_stack));
                        }
                    } else {
                        return Err(RuntimeError::new("stack underflow while comparing!".into(), call_stack));
                    }

                },
                Command::LE => {
                    if let (Some(right), Some(left)) = (env.stack.pop(), env.stack.pop()) {
                        if let (StackSlot::Number(r), StackSlot::Number(l)) = (right, left) {
                            env.stack.push(StackSlot::Number(if l <= r { 1.0 } else { 0.0 }));
                        } else {
                            return Err(RuntimeError::new("arithmetic is only supported for numbers".into(), call_stack));
                        }
                    } else {
                        return Err(RuntimeError::new("stack underflow while comparing!".into(), call_stack));
                    }
                },
                Command::GT => {
                    if let (Some(right), Some(left)) = (env.stack.pop(), env.stack.pop()) {
                        if let (StackSlot::Number(r), StackSlot::Number(l)) = (right, left) {
                            env.stack.push(StackSlot::Number(if l > r { 1.0 } else { 0.0 }));
                        } else {
                            return Err(RuntimeError::new("arithmetic is only supported for numbers".into(), call_stack));
                        }
                    } else {
                        return Err(RuntimeError::new("stack underflow while comparing!".into(), call_stack));
                    }
                },
                Command::GE => {
                    if let (Some(right), Some(left)) = (env.stack.pop(), env.stack.pop()) {
                        if let (StackSlot::Number(r), StackSlot::Number(l)) = (right, left) {
                            env.stack.push(StackSlot::Number(if l >= r { 1.0 } else { 0.0 }));
                        } else {
                            return Err(RuntimeError::new("arithmetic is only supported for numbers".into(), call_stack));
                        }
                    } else {
                        return Err(RuntimeError::new("stack underflow while comparing!".into(), call_stack));
                    }
                },
                Command::EQ => {
                    if let (Some(right), Some(left)) = (env.stack.pop(), env.stack.pop()) {
                        match (left, right) {
                            (StackSlot::Number(r), StackSlot::Number(l)) =>
                                env.stack.push(StackSlot::Number(if l == r { 1.0 } else { 0.0 })),
                            (StackSlot::String(r), StackSlot::String(l)) =>
                                env.stack.push(StackSlot::Number(if l == r { 1.0 } else { 0.0 })),
                            (StackSlot::String(_), StackSlot::Number(_)) =>
                                env.stack.push(StackSlot::Number(0.0)),
                            (StackSlot::Number(_), StackSlot::String(_)) =>
                                env.stack.push(StackSlot::Number(0.0)),
                            _ => {
                                return Err(RuntimeError::new("equal is only supported for Strings and Numbers".into(), call_stack));
                            }
                        }
                    } else {
                        return Err(RuntimeError::new("stack underflow while comparing!".into(), call_stack));
                    }
                },
                Command::NE => {
                    if let (Some(right), Some(left)) = (env.stack.pop(), env.stack.pop()) {
                        match (left, right) {
                            (StackSlot::Number(r), StackSlot::Number(l)) =>
                                env.stack.push(StackSlot::Number(if l != r { 1.0 } else { 0.0 })),
                            (StackSlot::String(r), StackSlot::String(l)) =>
                                env.stack.push(StackSlot::Number(if l != r { 1.0 } else { 0.0 })),
                            (StackSlot::String(_), StackSlot::Number(_)) =>
                                env.stack.push(StackSlot::Number(1.0)),
                            (StackSlot::Number(_), StackSlot::String(_)) =>
                                env.stack.push(StackSlot::Number(1.0)),
                            _ => {
                                return Err(RuntimeError::new("not equal is only supported for Strings and Numbers".into(), call_stack));
                            }
                        }
                    } else {
                        return Err(RuntimeError::new("stack underflow while comparing!".into(), call_stack));
                    }
                },
                Command::Not => {
                    if let Some(ss) = env.stack.pop() {
                        if let StackSlot::Number(n) = ss {
                            env.stack.push(StackSlot::Number(
                                if n == 0.0 {
                                    1.0
                                } else {
                                    0.0
                                }
                            ));
                        } else {
                            return Err(RuntimeError::new("negation is only supported for Numbers".into(), call_stack));
                        }
                    } else {
                        return Err(RuntimeError::new("stack underflow while negating".into(), call_stack));
                    }
                },
                Command::Dup => {
                    env.stack.push(env.stack.stack[env.stack.stack.len() - 1].clone());
                },
                Command::Swap => {
                    if let (Some(top), Some(bot)) = (env.stack.pop(), env.stack.pop()) {
                        env.stack.push(top);
                        env.stack.push(bot);
                    } else {
                        return Err(RuntimeError::new("stack underflow while swapping".into(), call_stack));
                    }
                },
                Command::Drop => {
                    env.stack.pop();
                },
                Command::Print => {
                    match env.stack.pop() {
                        Some(slot) => match slot {
                            StackSlot::Number(n) => log(&format!("{}", n)),
                            StackSlot::String(s) => log(&format!("{}", s.replace("\\n", "\n"))),
                            StackSlot::Reference(r, p) => log(&format!("{} -> {}", r, p)),
                            StackSlot::Code(_) => {}
                        },
                        None => println!("Stack underflow!")
                    };
                },
                Command::ArrowPut => {
                    let value: Command;
                    match env.stack.pop() {
                        Some(StackSlot::Number(n)) => value = Command::Pushn(n),
                        Some(StackSlot::String(s)) => value = Command::Pushs(s),
                        Some(StackSlot::Reference(r, offset)) => value = Command::Reference(String::from("@") + r.as_ref(), offset),
                        Some(StackSlot::Code(_)) => return Err(RuntimeError::new("Code on stack unsupported!".into(), call_stack)),
                        None => return Err(RuntimeError::new("stack underflow for arrow expression".into(), call_stack)),
                        _ => return Err(RuntimeError::new("expected value for arrow expression".into(), call_stack))
                    };

                    if let Command::Reference(name, offset) = env.program[env.idx + 1].clone() {
                        if let Ok(pos) = Environment::resolve_reference(&env.definitions, name.split("@").collect::<Vec<&str>>()[1].into()) {
                            env.program[pos + 1 + offset] = value.clone();
                            env.idx += 1;
                        }
                        else {
                            return Err(RuntimeError::new(format!("no such symbol: `{}`", name), call_stack));
                        }
                    } else {
                        return Err(RuntimeError::new("reference required for arrow put".into(), call_stack));
                    }
                },
                Command::Put => {
                    if let Some(StackSlot::Reference(name, offset)) = env.stack.pop() {
                        let pos = env.definitions[&name] + offset;
                        match env.stack.pop() {
                            Some(StackSlot::Number(n)) => env.program[pos + 1] = Command::Pushn(n),
                            Some(StackSlot::String(s)) => env.program[pos + 1] = Command::Pushs(s),
                            Some(StackSlot::Reference(r, offset)) => env.program[pos + 1] = Command::Reference(String::from("@") + r.as_ref(), offset),
                            Some(StackSlot::Code(_)) => {}
                            None => {
                                return Err(RuntimeError::new("value required for put".into(), call_stack));
                            }
                        };
                    } else {
                        return Err(RuntimeError::new("reference required for put".into(), call_stack));
                    }
                },
                Command::Get => {
                    if let Some(StackSlot::Reference(name, offset)) = env.stack.pop() {
                        let pos = env.definitions[&name] + offset;
                            match env.program.get(pos + 1) {
                                Some(Command::Pushn(n)) => env.stack.push(StackSlot::Number(*n)),
                                Some(Command::Pushs(s)) => env.stack.push(StackSlot::String(s.clone())),
                                _ => return Err(RuntimeError::new("value required for get".into(), call_stack))
                            }
                    } else {
                        return Err(RuntimeError::new("reference required for get".into(), call_stack));
                    }
                }
                Command::Pull => {
                    if let StackSlot::Number(n) = env.stack.pop().unwrap() {
                        if n.is_sign_positive() && n.floor() == n {
                            env.stack.push(env.stack.stack[n as usize].clone())
                        } else if n.is_sign_negative() && n.floor() == n {
                            env.stack.push(env.stack.stack[(env.stack.stack.len() as isize + n as isize) as usize].clone())
                        } else {
                            return Err(RuntimeError::new("expected integer for pull".into(), call_stack));
                        }
                    } else {
                        return Err(RuntimeError::new("expected integer for pull".into(), call_stack));
                    }
                },
                Command::Reference(s, offset) => {
                    let name: &str = &s[1..];
                    if env.definitions.contains_key(name) {
                        env.stack.push(StackSlot::Reference(String::from(name), *offset));
                    } else {
                        return Err(RuntimeError::new(format!("no such symbol: `{}`", name), call_stack));
                    }
                },
                Command::AddressOf => {
                    if let StackSlot::String(name) = env.stack.pop().unwrap() {
                        let s: &str = name.as_ref();
                        if env.definitions.contains_key(s) {
                            env.stack.push(StackSlot::Reference(String::from(s), 0));
                        } else {
                            return Err(RuntimeError::new(format!("no such symbol: `{}`", s), call_stack));

                        }
                    } else {
                        return Err(RuntimeError::new("string required".into(), call_stack));
                    }
                },
                Command::SubProg => {
                    let mut sub_env = Environment::new(env.program.clone());
                    sub_env.idx = env.idx + 1;
                    sub_env.prefix = env.prefix.clone();
                    sub_env.definitions = env.definitions.clone();

                    match run(&mut sub_env) {
                        Ok(result) => {
                            env.idx = result.idx;
                            if let Some(stack_slot) = result.stack.stack.last() {
                                env.stack.push(stack_slot.clone());
                            }
                        },
                        Err(err) =>
                            println!("error in sub: {}", err)
                    }
                },
                Command::Run => {
                    if let StackSlot::Code(prog) = env.stack.pop().unwrap() {
                        match run(&mut Environment::new(prog)) {
                            Ok(new_env) => for item in new_env.stack.stack {
                                env.stack.push(item);
                            },
                            Err(e) => return Err(RuntimeError::new(format!("lambda error: {}", e), call_stack))
                        }
                    } else {
                        return Err(RuntimeError::new("run requires lambda on stack".into(), call_stack));
                    }
                }
                Command::Lambda => {
                    let lambda_name = format!("__lambda_{}", env.lambda_counter);

                    env.define_new(lambda_name.clone());
                    env.stack.push(StackSlot::Reference(String::from(&lambda_name), 0));

                    // the would-be name will be dropped
                    env.stack.push(StackSlot::Number(-1.0));
                    env.lambda_counter += 1;
                    execute = false;
                }
                Command::End => {
                    break;
                }
                Command::PrintStack => println!("{:?}", env.stack.stack),
                Command::JSEval => if let StackSlot::String(s) = env.stack.pop().unwrap() {
                        env.stack.push(StackSlot::Number(eval(s.as_ref())));
                    }
                    else {
                        return Err(RuntimeError::new("javascript needs to be a string".into(), call_stack));
                    },
                Command::Placeholder => {
                    return Err(RuntimeError::new(format!("encountered placeholder"), call_stack));
                },
                _ => {}
            }
        }

        env.idx += 1;
    }
    Ok(
        Environment::from(env.clone())
    )
}

fn lexer(program: String) -> Environment {
    let mut commands: Vec<Command> = Vec::new();
    let mut idx = 0;
    let comment = Regex::new(r"(?m)//.*$").unwrap();

    let mut preprocessed = program;
    preprocessed = preprocessed.replace("(", " ( ");
    preprocessed = preprocessed.replace(")", " ) ");
    preprocessed = preprocessed.replace("$", " get ");
    preprocessed = comment.replace_all(preprocessed.as_ref(), "").to_string();

    let prog: Vec<&str> = preprocessed
        .split_whitespace()
        .collect();

    while idx < prog.len() {
        let next: Command =
            match prog[idx].as_ref() {
                ";" =>
                    Command::Nop,
                "[" =>
                    Command::Nop,
                "]" =>
                    Command::Nop,
                "include" =>
                    Command::Include,
                "STACK" =>
                    Command::PrintStack,
                "__jseval" =>
                    Command::JSEval,
                "{" =>
                    Command::Define(Visibility::Public),
                "}" =>
                    Command::EndDefine,
                "is" =>
                    Command::Define(Visibility::Public),
                "=" =>
                    Command::Define(Visibility::Public),
                "priv" =>
                    Command::Define(Visibility::Private),
                "in" =>
                    Command::EndDefine,
                "return" =>
                    Command::Return,
                "jump" =>
                    Command::Jmp,
                "jump?" =>
                    Command::JmpIf,
                "not" =>
                    Command::Not,
                "dup" =>
                    Command::Dup,
                "swap" =>
                    Command::Swap,
                "drop" =>
                    Command::Drop,
                "put" =>
                    Command::Put,
                "get" =>
                    Command::Get,
                "pull" =>
                    Command::Pull,
                "->" =>
                    Command::ArrowPut,
                "sub" =>
                    Command::SubProg,
                "lambda" =>
                    Command::Lambda,
                "run" =>
                    Command::Run,
                "end" =>
                    Command::End,
                "+" =>
                    Command::Add,
                "-" =>
                    Command::Sub,
                "*" =>
                    Command::Mul,
                "/" =>
                    Command::Div,
                "%" =>
                    Command::Mod,
                "<" =>
                    Command::LT,
                "<=" =>
                    Command::LE,
                ">" =>
                    Command::GT,
                ">=" =>
                    Command::GE,
                "==" =>
                    Command::EQ,
                "!=" =>
                    Command::NE,
                "addr" =>
                    Command::AddressOf,
                "print" =>
                    Command::Print,
                "_" => 
                    Command::Return,
                s if s == "\\space" => {
                    Command::Pushs(String::from(" "))
                },
                s if String::from(s).starts_with("\"") => {
                    let mut string = String::from(s);

                    while !prog[idx].ends_with("\"") || prog[idx].ends_with("\\\"") {
                        idx += 1;
                        string.push(' ');
                        string.push_str(prog[idx].as_ref());
                    }

                    Command::Pushs(String::from(&string)
                        .get(1..string.len() - 1).unwrap()
                        .replace("\\\"", "\"")
                        .replace("\\n", "\n"))
                },
                s if s.chars().nth(0).unwrap() == '@' => {
                    let mut last_idx = s.len();
                    let mut jumps = vec![];
                    while s[0..last_idx].ends_with("!") {
                        last_idx -= 1;
                        jumps.push(Command::Jmp);
                    }
                    commands.push(Command::Reference(String::from(&s[0..last_idx]), 0));
                    commands.append(&mut jumps);
                    Command::Nop
                },
                s if s.chars().nth(0).unwrap() == '_' => {
                    let n = (&s[1..]).parse::<usize>();
                    if let Ok(v) = n {
                        for i in 0..v-1 {
                            commands.push(Command::Return);
                        }
                    }
                    Command::Return
                },
                s if s.parse::<f64>().is_ok() =>
                    Command::Pushn(s.parse::<f64>().unwrap()),
                s if String::from(s).starts_with("(") => {
                    let stuff = arithparser::parse(&prog, &mut idx);
                    for com in stuff[..stuff.len() - 1].iter() {
                        commands.push(com.clone());
                    }
                    stuff[stuff.len() - 1].clone()
                }
                s =>
                    Command::Pushs(String::from(s))
            };
        match next {
            Command::Nop => {},
             n => commands.push(n)
        }
        idx += 1;
    }

    Environment::new(commands)
}

fn main() {
    println!("nothing here!")
}

#[wasm_bindgen]
pub fn run_string(input: &str) -> String {
    format!("{}", {
        let mut env = ENV.lock().unwrap();
        let idx = env.program.len();
        env.program.append(&mut lexer(input.to_string()).program);
        env.idx = idx;
        match run(&mut env) {
            Ok(env) => env.stack,
            Err(msg) => {
                error(&format!(
                    "Notherlang Error:\nat: {:#?}\nstack: {:#?}\n{}", 
                    &env.program[usize::max(0, env.idx - 1)..usize::min(env.program.len() - 1, env.idx + 2)],
                    env.stack.stack,
                    msg));
                Stack { stack: vec![] }
            }
        }
   })
}
