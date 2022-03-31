mod arithparser;

use std::fs;
use std::collections::HashMap;
use std::fmt;
use regex::Regex;
use std::convert::TryInto;

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Private
}

#[derive(Debug, Clone)]
pub enum SourceReference {
    Visible(String),
    Invisible
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Pushn(f64),
    Pushs(String),
    Define(Visibility),
    EndDefine,
    Jmp,
    JmpIf,
    LoopIf,
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
    NamedReference(String, usize),
    AbsoluteReference(usize),
    Print,
    Not,
    Dup,
    Swap,
    Drop,
    Put,
    Get,
    ArrowPut,
    AddressOf,
    Lambda,
    Return,
    Pull,

    Nop,
    Include,
    PrintStack,
    Bytes,
    Placeholder
}

#[derive(Debug, Clone)]
pub enum StackSlot {
    Number(f64),
    String(String),
    NamedReference(String, usize),
    AbsoluteReference(usize)
}

#[derive(Clone, Debug)]
pub struct Stack {
    stack: Vec<StackSlot>
}

#[derive(Debug)]
pub struct RuntimeError {
    msg: String,
    call_stack: Vec<(usize, String, usize)>,
    env: Environment
}

#[derive(Clone, Debug)]
pub struct Environment {
    prefix: Vec<String>,
    pub stack: Stack,
    definitions: HashMap<String, usize>,
    program: Vec<Command>,
    source: Vec<SourceReference>,
    idx: usize,
    pub execute: bool,
    level: u32
}

impl Environment {
    pub fn new(program: Vec<Command>, source: Vec<SourceReference>) -> Self {
        Environment {
            prefix: Vec::new(),
            stack: Stack { stack: Vec::new() },
            definitions: HashMap::new(),
            program: program,
            source: source,
            idx: 0,
            execute: true,
            level: 0
        }
    }

    fn from(from: Environment) -> Self {
        Environment {
            prefix: from.prefix,
            stack: from.stack,
            definitions: from.definitions,
            program: from.program,
            source: from.source,
            idx: from.idx,
            execute: from.execute,
            level: from.level
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
        self.prefix.push(s);
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

    fn pop_number(&mut self) -> Option<f64> {
        match self.stack.pop() {
            Some(StackSlot::Number(n)) => Some(n),
            _ => None
        }
    }

    fn pop_string(&mut self) -> Option<String> {
        match self.stack.pop() {
            Some(StackSlot::String(s)) => Some(s),
            _ => None
        }
    }

    fn pop_reference(&mut self) -> Option<(String, usize)> {
        match self.stack.pop() {
            Some(StackSlot::NamedReference(s, n)) => Some((s, n)),
            _ => None
        }
    }
}

impl RuntimeError {
    fn new(msg: String, call_stack: &Vec<(usize, String, usize)>, env: &Environment) -> Self {
        RuntimeError {
            msg: msg, call_stack: call_stack.to_vec(), env: env.clone()
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let call_info = self.call_stack.iter().map(|f| f.1.clone()).collect::<Vec<String>>();
        let info_start_idx = usize::max(0, self.env.idx - 10);
        let info_end_idx = usize::min(self.env.source.len(), self.env.idx + 10);
        write!(f,
            "RuntimeError: {}\ncallstack: {:#?}\n",
            self.msg,
            call_info);
        write!(f, "Around here: {}", self.env.source[info_start_idx..info_end_idx].iter()
            .map(|sr| match sr {
                SourceReference::Visible(s) => format!(" {}", s),
                SourceReference::Invisible => String::new(),
            }).collect::<String>()
        )
    }
}

impl fmt::Display for Stack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.stack.iter().map(|e| format!("{:?}\n", e)).fold(String::default(), |acc, e| acc + &e))
    }
}

fn run(env: &mut Environment) -> Result<(), RuntimeError> {
    let mut call_stack: Vec<(usize, String, usize)> = Vec::new();

    while env.idx < env.program.len() { 
        match &env.program[env.idx] {
            Command::Define(v) => {
                if let Visibility::Public = v {
                    if let Command::Pushs(string) = env.program[env.idx - 1].clone() {
                        env.define_new(string);
                        env.execute = false;
                        env.level += 1;
                    }
                    else {
                        return Err(RuntimeError::new("public define needs a label".into(), &call_stack, &env));
                    }
                } else if env.execute {
                    if let Some(StackSlot::String(string)) = env.stack.pop() {
                        println!("priv");
                        env.define_new(string);
                        env.execute = false;
                        env.level += 1;
                    }
                    else {
                        return Err(RuntimeError::new("string required for private define".into(), &call_stack, &env));
                    }
                }
            },
            Command::Lambda => {
                env.level += 1;
            },
            Command::NamedReference(name, offset) => {
                // if the address starts with `::` the scope will be inferred
                if name.starts_with("@::") {
                    let mut full_name = None;
                    for i in 0..env.prefix.len() {
                        let test_name = format!("@{}{}", env.prefix[0..env.prefix.len() - i].join("::"), &name[1..]);
                        if env.definitions.contains_key(&test_name[1..]) {
                            full_name = Some(test_name);
                            break;
                        }
                    }
                    env.program[env.idx] = Command::NamedReference(
                        full_name
                            .ok_or_else(|| RuntimeError::new(format!("no such symbol: `{}`", name), &call_stack, &env))?,
                        *offset
                    );
                }
            }
            _ => {}
        }

        if !env.execute {
            match &env.program[env.idx] {
                Command::Lambda => {
                    env.prefix.push("lambda".into())
                },
                Command::EndDefine => {
                    env.prefix.pop();
                    env.level -= 1;
                    if env.level < 1 {
                        env.stack.pop();

                        env.execute = true;
                        env.idx += 1;
                        continue;
                    }
                },
                _ => {}
            }
        }

        if env.execute {
            match &env.program[env.idx] {
                Command::Nop => { },
                Command::Include => {
                    let filename = env.stack.pop_string()
                        .ok_or_else(|| RuntimeError::new("expected file name for include".into(), &call_stack, &env))?;
                    let content = fs::read_to_string(filename.clone())
                        .or_else(|_err| fs::read_to_string(format!("lib/{}", filename)))
                        .or_else(|err| Err(RuntimeError::new(format!("unable to read include file: {}", err), &call_stack, &env)))?;

                    let result = lexer(content.to_string());
                    let tokens = result.program;
                    let source = result.source;

                    for (token, source_ref) in tokens.iter().zip(source.iter()).rev() {
                        env.program.insert(env.idx + 1, token.clone());
                        env.source.insert(env.idx + 1, source_ref.clone());
                    }
                }
                Command::Pushn(n) => env.stack.push(StackSlot::Number(*n)),
                Command::Pushs(s) => {
                    env.stack.push(StackSlot::String(s.clone()))
                },
                Command::EndDefine | Command::Return => if call_stack.len() > 0 {
                    env.idx = call_stack.pop().unwrap().0 as usize;
                },
                Command::LoopIf => {
                    let reference_name = call_stack.last()
                        .ok_or_else(|| RuntimeError::new("can't use `loop?` on toplevel".into(), &call_stack, &env))?;

                    let position = reference_name.2;

                    let n = env.stack.pop_number()
                        .ok_or_else(|| RuntimeError::new("expected number for a loop".into(), &call_stack, &env))?;
                    if n != 0.0 {
                        env.idx = position;
                    }
                },
                Command::JmpIf => {
                    let reference = env.stack.pop();
                    let value = env.stack.pop_number()
                        .ok_or_else(|| RuntimeError::new("expected number for a conditional jump".into(), &call_stack, &env))?;

                    if value != 0.0 {
                        match reference {
                            Some(StackSlot::NamedReference(n, offset)) => {
                                if env.definitions.contains_key(&n) {
                                    let next_idx = env.definitions[&n] + offset;
                                    call_stack.push((env.idx, n.clone(), next_idx));
                                    env.idx = next_idx;
                                } else {
                                    return Err(RuntimeError::new("reference not found in definitions for `jump?`".into(), &call_stack, &env));
                                }
                            },
                            Some(StackSlot::AbsoluteReference(position)) => {
                                call_stack.push((env.idx, format!("->{}", position), position));
                                env.idx = position;
                            },
                            _ => {
                                return Err(RuntimeError::new("expected reference for a jump".into(), &call_stack, &env));
                            }
                        }
                    }
                }
                Command::Jmp => {
                    match env.stack.pop() {
                        Some(StackSlot::NamedReference(n, offset)) => {
                            if env.definitions.contains_key(&n) {
                                let next_idx = env.definitions[&n] + offset;
                                call_stack.push((env.idx, n.clone(), next_idx));
                                env.idx = next_idx;
                            } else {
                                return Err(RuntimeError::new("reference not found in definitions `jump`".into(), &call_stack, &env));
                            }
                        },
                        Some(StackSlot::AbsoluteReference(position)) => {
                            call_stack.push((env.idx, format!("->{}", position), position));
                            env.idx = position;
                        },
                        _ => {
                            return Err(RuntimeError::new("expected reference for a jump".into(), &call_stack, &env));
                        }
                    }
                }
                Command::Add => {
                    if let (Some(right), Some(left)) = (env.stack.pop(), env.stack.pop()) {
                        if let (StackSlot::Number(r), StackSlot::Number(l)) = (&right, &left) {
                            env.stack.push(StackSlot::Number(l + r));
                        } else if let (StackSlot::String(r), StackSlot::Number(l)) = (&right, &left) {
                            if let Ok(character) = TryInto::<char>::try_into(*l as u8) {
                                env.stack.push(StackSlot::String(
                                    format!("{}{}", character, r)
                                ));
                            } else {
                                return Err(RuntimeError::new(format!("{} is no a valid ascii character", l), &call_stack, &env));
                            }
                        } else if let (StackSlot::Number(r), StackSlot::String(l)) = (&right, &left) {
                            if let Ok(character) = TryInto::<char>::try_into(*r as u8) {
                                env.stack.push(StackSlot::String(
                                    format!("{}{}", l, character)
                                ));
                            } else {
                                return Err(RuntimeError::new(format!("{} is no a valid ascii character", r), &call_stack, &env));
                            }
                        } else if let (StackSlot::String(r), StackSlot::String(l)) = (&right, &left) {
                            env.stack.push(StackSlot::String(
                                format!("{}{}", l, r)
                            ));
                        } else if let (StackSlot::Number(r), StackSlot::NamedReference(name, l)) = (&right, &left) {
                            env.stack.push(StackSlot::NamedReference(
                                name.clone(), *r as usize + l
                            ));
                        } else if let (StackSlot::Number(r), StackSlot::AbsoluteReference(position)) = (&right, &left) {
                            env.stack.push(StackSlot::AbsoluteReference(
                                *r as usize + position
                            ));
                        } else {
                            return Err(RuntimeError::new("add operator only supported for numbers or strings".into(), &call_stack, &env));
                        }
                    } else {
                        return Err(RuntimeError::new("stack underflow while adding!".into(), &call_stack, &env));
                    }
                },
                Command::Sub => {
                    match (env.stack.pop(), env.stack.pop()) {
                        (Some(StackSlot::Number(r)), Some(StackSlot::Number(l))) => {
                            env.stack.push(StackSlot::Number(l - r));
                        },
                        (Some(StackSlot::NamedReference(name, offset)), Some(StackSlot::AbsoluteReference(position))) => {
                            let r = env.definitions.get(&name)
                                .ok_or_else(|| RuntimeError::new("reference not found in definitions for subtraction".into(), &call_stack, &env))? + offset;

                            env.stack.push(StackSlot::Number((position - r) as f64));
                        },
                        (Some(StackSlot::AbsoluteReference(position)), Some(StackSlot::NamedReference(name, offset))) => {
                            let l = env.definitions.get(&name)
                                .ok_or_else(|| RuntimeError::new("reference not found in definitions for subtraction".into(), &call_stack, &env))? + offset;

                            env.stack.push(StackSlot::Number((l - position) as f64));
                        },
                        (Some(StackSlot::NamedReference(rname, roffset)), Some(StackSlot::NamedReference(lname, loffset))) => {
                            let r = env.definitions.get(&rname)
                                .ok_or_else(|| RuntimeError::new("reference not found in definitions for subtraction".into(), &call_stack, &env))? + roffset;
                            let l = env.definitions.get(&lname)
                                .ok_or_else(|| RuntimeError::new("reference not found in definitions for subtraction".into(), &call_stack, &env))? + loffset;

                            env.stack.push(StackSlot::Number((l - r) as f64));
                        },
                        (Some(StackSlot::AbsoluteReference(r)), Some(StackSlot::AbsoluteReference(position))) => {
                            env.stack.push(StackSlot::Number((position - r) as f64));
                        },
                        _ => return Err(RuntimeError::new("arithmetic is only supported for numbers".into(), &call_stack, &env))
                    };
                },
                Command::Mul => {
                    if let (Some(right), Some(left)) = (env.stack.pop(), env.stack.pop()) {
                        if let (StackSlot::Number(r), StackSlot::Number(l)) = (right, left) {
                            env.stack.push(StackSlot::Number(l * r));
                        } else {
                            return Err(RuntimeError::new("arithmetic is only supported for numbers".into(), &call_stack, &env));
                        }
                    } else {
                        return Err(RuntimeError::new("stack underflow while multiplying!".into(), &call_stack, &env));
                    }
                },
                Command::Div => {
                    if let (Some(right), Some(left)) = (env.stack.pop(), env.stack.pop()) {
                        if let (StackSlot::Number(r), StackSlot::Number(l)) = (right, left) {
                            env.stack.push(StackSlot::Number(l / r));
                        } else {
                            return Err(RuntimeError::new("arithmetic is only supported for numbers".into(), &call_stack, &env));
                        }
                    } else {
                        return Err(RuntimeError::new("stack underflow while dividing!".into(), &call_stack, &env));
                    }
                },
                Command::Mod => {
                    if let (Some(right), Some(left)) = (env.stack.pop(), env.stack.pop()) {
                        if let (StackSlot::Number(r), StackSlot::Number(l)) = (right, left) {
                            env.stack.push(StackSlot::Number(l % r));
                        } else {
                            return Err(RuntimeError::new("arithmetic is only supported for numbers".into(), &call_stack, &env));
                        }
                    } else {
                        return Err(RuntimeError::new("stack underflow in modulo operation!".into(), &call_stack, &env));
                    }
                },
                Command::LT => {
                    if let (Some(right), Some(left)) = (env.stack.pop(), env.stack.pop()) {
                        if let (StackSlot::Number(r), StackSlot::Number(l)) = (right, left) {
                            env.stack.push(StackSlot::Number(if l < r { 1.0 } else { 0.0 }));
                        } else {
                            return Err(RuntimeError::new("arithmetic is only supported for numbers".into(), &call_stack, &env));
                        }
                    } else {
                        return Err(RuntimeError::new("stack underflow while comparing!".into(), &call_stack, &env));
                    }

                },
                Command::LE => {
                    if let (Some(right), Some(left)) = (env.stack.pop(), env.stack.pop()) {
                        if let (StackSlot::Number(r), StackSlot::Number(l)) = (right, left) {
                            env.stack.push(StackSlot::Number(if l <= r { 1.0 } else { 0.0 }));
                        } else {
                            return Err(RuntimeError::new("arithmetic is only supported for numbers".into(), &call_stack, &env));
                        }
                    } else {
                        return Err(RuntimeError::new("stack underflow while comparing!".into(), &call_stack, &env));
                    }
                },
                Command::GT => {
                    if let (Some(right), Some(left)) = (env.stack.pop(), env.stack.pop()) {
                        if let (StackSlot::Number(r), StackSlot::Number(l)) = (right, left) {
                            env.stack.push(StackSlot::Number(if l > r { 1.0 } else { 0.0 }));
                        } else {
                            return Err(RuntimeError::new("arithmetic is only supported for numbers".into(), &call_stack, &env));
                        }
                    } else {
                        return Err(RuntimeError::new("stack underflow while comparing!".into(), &call_stack, &env));
                    }
                },
                Command::GE => {
                    if let (Some(right), Some(left)) = (env.stack.pop(), env.stack.pop()) {
                        if let (StackSlot::Number(r), StackSlot::Number(l)) = (right, left) {
                            env.stack.push(StackSlot::Number(if l >= r { 1.0 } else { 0.0 }));
                        } else {
                            return Err(RuntimeError::new("arithmetic is only supported for numbers".into(), &call_stack, &env));
                        }
                    } else {
                        return Err(RuntimeError::new("stack underflow while comparing!".into(), &call_stack, &env));
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
                                env.stack.push(StackSlot::Number(0.0))
                            }
                        }
                    } else {
                        return Err(RuntimeError::new("stack underflow while comparing!".into(), &call_stack, &env));
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
                                env.stack.push(StackSlot::Number(0.0))
                            }
                        }
                    } else {
                        return Err(RuntimeError::new("stack underflow while comparing!".into(), &call_stack, &env));
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
                            return Err(RuntimeError::new("negation is only supported for Numbers".into(), &call_stack, &env));
                        }
                    } else {
                        return Err(RuntimeError::new("stack underflow while negating".into(), &call_stack, &env));
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
                        return Err(RuntimeError::new("stack underflow while swapping".into(), &call_stack, &env));
                    }
                },
                Command::Drop => {
                    env.stack.pop();
                },
                Command::Print => {
                    match env.stack.pop() {
                        Some(slot) => match slot {
                            StackSlot::Number(n) => print!("{}", n),
                            StackSlot::String(s) => print!("{}", s.replace("\\n", "\n")),
                            StackSlot::NamedReference(r, p) => print!("@{}+{}", r, p),
                            StackSlot::AbsoluteReference(p) => print!("@{}", p)
                        },
                        None => println!("Stack underflow!")
                    };
                },
                Command::ArrowPut => {
                    let value: Command;
                    match env.stack.pop() {
                        Some(StackSlot::Number(n)) => value = Command::Pushn(n),
                        Some(StackSlot::String(s)) => value = Command::Pushs(s),
                        Some(StackSlot::NamedReference(r, offset)) => value = Command::NamedReference(String::from("@") + r.as_ref(), offset),
                        Some(StackSlot::AbsoluteReference(position)) => value = Command::AbsoluteReference(position),
                        None => return Err(RuntimeError::new("stack underflow for arrow expression".into(), &call_stack, &env))
                    };

                    if let Command::NamedReference(name, offset) = env.program[env.idx + 1].clone() {
                        if let Ok(pos) = Environment::resolve_reference(&env.definitions, name.split("@").collect::<Vec<&str>>()[1].into()) {
                            env.program[pos + 1 + offset] = value.clone();
                            env.idx += 1;
                        }
                        else {
                            return Err(RuntimeError::new(format!("no such symbol: `{}`", name), &call_stack, &env));
                        }
                    } else {
                        return Err(RuntimeError::new("reference required for arrow put".into(), &call_stack, &env));
                    }
                },
                Command::Put => {
                    let pos = match env.stack.pop() {
                        Some(StackSlot::NamedReference(name, offset)) => {
                            let base = env.definitions.get(&name)
                                .ok_or_else(|| RuntimeError::new(format!("no such symbol: `{}`", name), &call_stack, &env))?;

                            base + offset
                        },
                        Some(StackSlot::AbsoluteReference(position)) => position,
                        _ => return Err(RuntimeError::new("reference required for put".into(), &call_stack, &env))
                    };

                    match env.stack.pop() {
                        Some(StackSlot::Number(n)) => env.program[pos + 1] = Command::Pushn(n),
                        Some(StackSlot::String(s)) => env.program[pos + 1] = Command::Pushs(s),
                        Some(StackSlot::NamedReference(r, offset)) => env.program[pos + 1] = Command::NamedReference(String::from("@") + r.as_ref(), offset),
                        Some(StackSlot::AbsoluteReference(position)) => env.program[pos + 1] = Command::AbsoluteReference(position),
                        None => {
                            return Err(RuntimeError::new("value required for put".into(), &call_stack, &env));
                        }
                    };
                },
                Command::Get => {
                    let pos = match env.stack.pop() {
                        Some(StackSlot::NamedReference(name, offset)) => {
                            let base = env.definitions.get(&name)
                                .ok_or_else(|| RuntimeError::new(format!("no such symbol: `{}`", name), &call_stack, &env))?;

                            base + offset
                        },
                        Some(StackSlot::AbsoluteReference(position)) => position,
                        _ => return Err(RuntimeError::new("reference required for get".into(), &call_stack, &env))
                    };

                    match env.program.get(pos + 1) {
                        Some(Command::Pushn(n)) => env.stack.push(StackSlot::Number(*n)),
                        Some(Command::Pushs(s)) => env.stack.push(StackSlot::String(s.clone())),
                        Some(Command::NamedReference(s, offset)) => env.stack.push(StackSlot::NamedReference(String::from(&s[1..]), *offset)),
                        Some(Command::AbsoluteReference(position)) => env.stack.push(StackSlot::AbsoluteReference(*position)),
                        _ => return Err(RuntimeError::new("value required for get".into(), &call_stack, &env))
                    }
                }
                Command::Pull => {
                    if let StackSlot::Number(n) = env.stack.pop().unwrap() {
                        if n.is_sign_positive() && n.floor() == n {
                            env.stack.push(env.stack.stack[n as usize].clone())
                        } else if n.is_sign_negative() && n.floor() == n {
                            env.stack.push(env.stack.stack[(env.stack.stack.len() as isize + n as isize) as usize].clone())
                        } else {
                            return Err(RuntimeError::new("expected integer for pull".into(), &call_stack, &env));
                        }
                    } else {
                        return Err(RuntimeError::new("expected integer for pull".into(), &call_stack, &env));
                    }
                },
                Command::NamedReference(s, offset) => {
                    let name: &str = &s[1..];
                    if env.definitions.contains_key(name) {
                        env.stack.push(StackSlot::NamedReference(String::from(name), *offset));

                        // replace with an absolute address to prevent excessive name lookups
                        let absolute_addess = env.definitions[name] + offset;
                        env.program[env.idx] = Command::AbsoluteReference(absolute_addess);
                    } else {
                        return Err(RuntimeError::new(format!("no such symbol: `{}`", name), &call_stack, &env));
                    }
                },
                Command::AbsoluteReference(position) => {
                    env.stack.push(StackSlot::AbsoluteReference(*position));
                },
                Command::AddressOf => {
                    if let StackSlot::String(name) = env.stack.pop().unwrap() {
                        let s: &str = name.as_ref();
                        if env.definitions.contains_key(s) {
                            env.stack.push(StackSlot::NamedReference(String::from(s), 0));
                        } else {
                            return Err(RuntimeError::new(format!("no such symbol: `{}`", s), &call_stack, &env));

                        }
                    } else {
                        return Err(RuntimeError::new("string required".into(), &call_stack, &env));
                    }
                },
                Command::Lambda => {
                    env.stack.push(StackSlot::AbsoluteReference(env.idx));

                    // the would-be name will be dropped
                    env.stack.push(StackSlot::Number(-1.0));
                    env.execute = false;
                }
                Command::PrintStack => println!("{:?}", env.stack.stack),
                Command::Placeholder => {
                    return Err(RuntimeError::new(format!("encountered placeholder"), &call_stack, &env));
                },
                Command::Bytes => if let StackSlot::String(s) = env.stack.pop().unwrap() {
                        for byte in s.as_bytes() {
                            env.stack.push(StackSlot::Number(*byte as f64));
                        }
                    }
                    else {
                        return Err(RuntimeError::new("needs a string to convert into number list".into(), &call_stack, &env));
                    },
                _ => {}
            }
        }

        env.idx += 1;
    }
    Ok(())
}


fn ends_with_whitespace(string: &str) -> bool {
    let last = string.chars().last();
    last == Some('\n') || last == Some('\t') || last == Some(' ')
}

fn lexer2(program: String) -> Environment {
    let mut commands: Vec<Command> = Vec::new();
    let mut source_ref: Vec<(usize, usize)> = Vec::new();

    let mut start_idx = 0;
    let mut end_idx = 1;

    while end_idx < program.len() {
        let mut next = match (&program[start_idx..end_idx], &program[end_idx..end_idx+1]) {
            ("include", " " | "\n") => Some(Command::Include),
            ("STACK", " " | "\n")   => Some(Command::PrintStack),
            ("{", " " | "\n")       => Some(Command::Define(Visibility::Public)),
            ("}", " " | "\n")       => Some(Command::EndDefine),
            ("is", " " | "\n")      => Some(Command::Define(Visibility::Public)),
            ("in", " " | "\n")      => Some(Command::EndDefine),
            ("jump", " " | "\n")    => Some(Command::Jmp),
            ("jump?", " " | "\n")   => Some(Command::JmpIf),
            ("loop?", " " | "\n")   => Some(Command::LoopIf),
            ("not", " " | "\n")     => Some(Command::Not),
            ("dup", " " | "\n")     => Some(Command::Dup),
            ("swap", " " | "\n")    => Some(Command::Swap),
            ("drop", " " | "\n")    => Some(Command::Drop),
            ("put", " " | "\n")     => Some(Command::Put),
            ("get", " " | "\n")     => Some(Command::Get),
            ("pull", " " | "\n")    => Some(Command::Pull),
            ("->", " " | "\n")      => Some(Command::ArrowPut),
            ("lambda", " " | "\n")  => Some(Command::Lambda),
            ("__bytes", " " | "\n") => Some(Command::Bytes),
            ("+", " " | "\n")       => Some(Command::Add),
            ("-", " " | "\n")       => Some(Command::Sub),
            ("*", " " | "\n")       => Some(Command::Mul),
            ("/", " " | "\n")       => Some(Command::Div),
            ("%", " " | "\n")       => Some(Command::Mod),
            ("<", " " | "\n")       => Some(Command::LT),
            ("<=", " " | "\n")      => Some(Command::LE),
            (">", " " | "\n")       => Some(Command::GT),
            (">=", " " | "\n")      => Some(Command::GE),
            ("==", " " | "\n")      => Some(Command::EQ),
            ("!=", " " | "\n")      => Some(Command::NE),
            ("addr", " " | "\n")    => Some(Command::AddressOf),
            ("print", " " | "\n")   => Some(Command::Print),
            ("_", " " | "\n")       => Some(Command::Return),
            ("\\space", " " | "\n") => Some(Command::Pushs(" ".into())),
            _ => None
        };
        let mut found = true;
        if next.is_none() {
            match &program[start_idx..end_idx] {
                s if s.chars().nth(0) == Some('"') && 
                     s.chars().last() == Some('"') => {
                    commands.push(Command::Pushs(program[start_idx + 1..end_idx - 1].into()));
                    source_ref.push((start_idx, end_idx));
                },
                s if s.chars().nth(0) == Some('@') && ends_with_whitespace(s) => {
                    let jumps: Vec<Command> = s[0..s.len() - 1].chars().rev()
                        .map_while(|c| match c {
                            '$' => Some(Command::Get),
                            '!' => Some(Command::Jmp),
                            '?' => Some(Command::JmpIf),
                            _ => None
                        }).collect();

                    commands.push(Command::NamedReference(String::from(&s[0..s.len() - jumps.len() - 1]), 0));
                    source_ref.push((start_idx, end_idx));
                    source_ref.append(&mut (0..jumps.len()).map(|_| (start_idx, end_idx)).collect());
                    commands.append(&mut jumps.into_iter().rev().collect());
                },
                s if s.chars().nth(0) == Some('_') && ends_with_whitespace(s) => {
                    let n = (&s[1..]).parse::<usize>();
                    if let Ok(v) = n {
                        for _i in 0..v {
                            commands.push(Command::Return);
                            source_ref.push((start_idx, end_idx));
                        }
                    }
                },
                s if s[0..s.len() - 1].parse::<f64>().is_ok() && ends_with_whitespace(s) => {
                    commands.push(Command::Pushn(s[0..s.len() - 1].parse::<f64>().unwrap()));
                    source_ref.push((start_idx, end_idx));
                },
                s if String::from(s).starts_with("(") => {
                    // TODO
                }
                s if ends_with_whitespace(s) => {
                    commands.push(Command::Pushs(String::from(&s[0..s.len() - 1])));
                    source_ref.push((start_idx, end_idx));
                }
                s if s.starts_with("//") && s.chars().last() == Some('\n') => {
                    // found
                }
                _ => {
                }
            }
        }

        if next.is_some() {
            source_ref.push((start_idx, end_idx));
        }

        if !next.is_none() || found {
            start_idx = end_idx;
        }
        end_idx += 1;

        if let Some(token) = next {
            commands.push(token);
        }

        while program.chars().nth(start_idx) == Some(' ') || program.chars().nth(start_idx) == Some('\n') {
            start_idx += 1;
            end_idx += 1;
        }
    }

    println!("source refs: {:?}", source_ref.iter().map(|(start, end)| &program[*start..*end]).collect::<Vec<&str>>());
    Environment::new(commands, vec![])
}

fn lexer(program: String) -> Environment {
    let mut commands: Vec<Command> = Vec::new();
    let mut source: Vec<SourceReference> = Vec::new();
    let mut idx = 0;
    let comment = Regex::new(r"(?m)//.*$").unwrap();

    let mut preprocessed = program;
    preprocessed = preprocessed.replace("(", " ( ");
    preprocessed = preprocessed.replace(")", " ) ");
    preprocessed = comment.replace_all(preprocessed.as_ref(), "").to_string();

    let prog: Vec<&str> = preprocessed
        .split_whitespace()
        .collect();

    while idx < prog.len() {
        let next: Command =
            match prog[idx].as_ref() {
                "include" =>
                    Command::Include,
                "STACK" =>
                    Command::PrintStack,
                "{" =>
                    Command::Define(Visibility::Public),
                "}" =>
                    Command::EndDefine,
                "is" =>
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
                "loop?" =>
                    Command::LoopIf,
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
                "lambda" =>
                    Command::Lambda,
                "__bytes" =>
                    Command::Bytes,
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
                    let jumps: Vec<Command> = s.chars().rev()
                        .map_while(|c| match c {
                            '$' => Some(Command::Get),
                            '!' => Some(Command::Jmp),
                            '?' => Some(Command::JmpIf),
                            _ => None
                        }).collect();

                    commands.push(Command::NamedReference(String::from(&s[0..s.len() - jumps.len()]), 0));
                    source.push(SourceReference::Invisible);
                    source.append(&mut (0..jumps.len()).map(|_| SourceReference::Invisible).collect());
                    commands.append(&mut jumps.into_iter().rev().collect());
                    Command::Nop
                },
                s if s.chars().nth(0).unwrap() == '_' => {
                    let n = (&s[1..]).parse::<usize>();
                    if let Ok(v) = n {
                        for _i in 0..v-1 {
                            commands.push(Command::Return);
                            source.push(SourceReference::Invisible);
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
                        source.push(SourceReference::Invisible);
                    }
                    stuff[stuff.len() - 1].clone()
                }
                s =>
                    Command::Pushs(String::from(s))
            };
        match next {
            Command::Nop => {},
             n => {
                commands.push(n);
                source.push(SourceReference::Visible(prog[idx].to_string()));
            }
        }
        idx += 1;
    }

    Environment::new(commands, source)
}


pub fn run_string(mut env: &mut Environment, input: &str) -> Result<(), RuntimeError> {
    let mut result = lexer(input.to_string());
    env.program.append(&mut result.program);
    env.source.append(&mut result.source);

    let res = run(&mut env);
    env.idx = env.program.len();

    res
}

