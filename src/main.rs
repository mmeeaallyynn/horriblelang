extern crate regex;

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::os::unix::io::FromRawFd;
use std::io::prelude::*;
use regex::Regex;

#[derive(Debug, Clone)]
enum Visibility {
	Public,
	Private
}

#[derive(Debug, Clone)]
enum Command {
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
	GT,
	GE,
	LT,
	LE,
	EQ,
	NE,
	Reference(String),
	Print,
	Not,
	Dup,
	Swap,
	Drop,
	Put,
	Push,
	AddressOf,

	Include,
	PrintStack
}

#[derive(Debug, Clone)]
enum StackSlot {
	Number(f64),
	String(String),
	Reference(String, usize)
}

struct Stack {
	stack: Vec<StackSlot>
}

impl Stack {
	fn push(&mut self, item: StackSlot) {
		self.stack.push(item);
	}

	fn pop(&mut self) -> Option<StackSlot> {
		self.stack.pop()
	}
}

fn run(mut program: Vec<Command>) {
	let mut prefix: Vec<String> = Vec::new();
	let mut execute = true;
	let mut level = 0;

	let mut stack = Stack { stack: Vec::new() };
	let mut call_stack: Vec<usize> = Vec::new();
	let mut definitions: HashMap<String, usize> = HashMap::new();
	let mut idx: usize = 0;

	while idx < program.len() {
		match &program[idx] {
			Command::Define(v) => {
				let mut define_new = |s: String| {
					prefix.push(s.clone());
					let name = prefix.join("::");

					level += 1;

					if definitions.contains_key(&name) {
						*definitions.get_mut(&name).unwrap() = idx;
					} else {
						definitions.insert(
							name, idx
						);
					}
				};

				if let Visibility::Public = v {
					if let Command::Pushs(string) = &program[idx - 1] { define_new(string.to_string()); execute = false; }
					else { panic!("public define needs a label") }
				} else if execute {
					if let StackSlot::String(string) = stack.pop().unwrap() { define_new(string.to_string()); execute = false; }
					else { panic!("string required for private define") }
				}
			}
			_ => {}
		}

		if !execute {
			match &program[idx] {
				Command::EndDefine => {
					prefix.pop();
					level -= 1;
					if level < 1 {
						stack.pop();

						execute = true;
						idx += 1;
					}
				},
				_ => {}
			}
		}

		if execute {
			match &program[idx] {
				Command::Include => {
					if let StackSlot::String(filename) = stack.pop().unwrap() {
						let comment = Regex::new(r"/\*.*\*/").unwrap();
						let mut file = File::open(&filename)
							.expect("include: file not found");

						let mut content = String::new();

						file.read_to_string(&mut content)
							.expect("unable to read file");

						let result = comment.replace_all(content.as_ref(), "");
						let mut tokens = lexer(result.to_string());
						tokens.reverse();

						for token in tokens.iter() {
							program.insert(idx + 1, token.clone());
						}
					} else {
						panic!("expected file name for include");
					}
				}
				Command::Pushn(n) => stack.push(StackSlot::Number(*n)),
				Command::Pushs(s) => {
					stack.push(StackSlot::String(s.clone()))
				},
				Command::EndDefine => if call_stack.len() > 0 {
					idx = call_stack.pop().unwrap() as usize;
				},
				Command::JmpIf => {
					if let StackSlot::Reference(n, _) = stack.pop().unwrap() {
						if let StackSlot::Number(f) = stack.pop().unwrap() {
							if f != 0.0 {
								call_stack.push(idx);
								idx = definitions[&n];
							}
						} else {
							panic!("expected number for a conditional jump")
						}
					} else {
						panic!("expected reference for a jump");
					}
				}
				Command::Jmp => {
					if let StackSlot::Reference(n, _) = stack.pop().unwrap() {
						call_stack.push(idx);
						idx = definitions[&n];
					} else {
						panic!("expected reference for a jump");
					}
				}
				Command::Add => {
					let right = stack.pop().unwrap();
					let left = stack.pop().unwrap();

					if let (StackSlot::Number(r), StackSlot::Number(l)) = (right, left) {
						stack.push(StackSlot::Number(l + r));
					} else {
						panic!("arithmetic is only supported for numbers");
					}
				},
				Command::Sub => {
					let right = stack.pop().unwrap();
					let left = stack.pop().unwrap();

					if let (StackSlot::Number(r), StackSlot::Number(l)) = (right, left) {
						stack.push(StackSlot::Number(l - r));
					} else {
						panic!("arithmetic is only supported for numbers");
					}
				},
				Command::Mul => {
					let right = stack.pop().unwrap();
					let left = stack.pop().unwrap();

					if let (StackSlot::Number(r), StackSlot::Number(l)) = (right, left) {
						stack.push(StackSlot::Number(l * r));
					} else {
						panic!("arithmetic is only supported for numbers");
					}
				},
				Command::Div => {
					let right = stack.pop().unwrap();
					let left = stack.pop().unwrap();

					if let (StackSlot::Number(r), StackSlot::Number(l)) = (right, left) {
						stack.push(StackSlot::Number(l / r));
					} else {
						panic!("arithmetic is only supported for numbers");
					}
				},
				Command::LT => {
					let right = stack.pop().unwrap();
					let left = stack.pop().unwrap();

					if let (StackSlot::Number(r), StackSlot::Number(l)) = (right, left) {
						stack.push(StackSlot::Number(if l < r { 1.0 } else { 0.0 }));
					} else {
						panic!("arithmetic is only supported for numbers");
					}
				},
				Command::LE => {
					let right = stack.pop().unwrap();
					let left = stack.pop().unwrap();

					if let (StackSlot::Number(r), StackSlot::Number(l)) = (right, left) {
						stack.push(StackSlot::Number(if l <= r { 1.0 } else { 0.0 }));
					} else {
						panic!("arithmetic is only supported for numbers");
					}
				},
				Command::GT => {
					let right = stack.pop().unwrap();
					let left = stack.pop().unwrap();

					if let (StackSlot::Number(r), StackSlot::Number(l)) = (right, left) {
						stack.push(StackSlot::Number(if l > r { 1.0 } else { 0.0 }));
					} else {
						panic!("arithmetic is only supported for numbers");
					}
				},
				Command::GE => {
					let right = stack.pop().unwrap();
					let left = stack.pop().unwrap();

					if let (StackSlot::Number(r), StackSlot::Number(l)) = (right, left) {
						stack.push(StackSlot::Number(if l >= r { 1.0 } else { 0.0 }));
					} else {
						panic!("arithmetic is only supported for numbers");
					}
				},
				Command::EQ => {
					let right = stack.pop().unwrap();
					let left = stack.pop().unwrap();

					match (left, right) {
						(StackSlot::Number(r), StackSlot::Number(l)) =>
							stack.push(StackSlot::Number(if l == r { 1.0 } else { 0.0 })),
						(StackSlot::String(r), StackSlot::String(l)) =>
							stack.push(StackSlot::Number(if l == r { 1.0 } else { 0.0 })),
						(StackSlot::String(_), StackSlot::Number(_)) =>
							stack.push(StackSlot::Number(0.0)),
						(StackSlot::Number(_), StackSlot::String(_)) =>
							stack.push(StackSlot::Number(0.0)),
						_ => panic!("equal is only supported for Strings and Numbers")
					}
				},
				Command::NE => {
					let right = stack.pop().unwrap();
					let left = stack.pop().unwrap();

					match (left, right) {
						(StackSlot::Number(r), StackSlot::Number(l)) =>
							stack.push(StackSlot::Number(if l != r { 1.0 } else { 0.0 })),
						(StackSlot::String(r), StackSlot::String(l)) =>
							stack.push(StackSlot::Number(if l != r { 1.0 } else { 0.0 })),
						(StackSlot::String(_), StackSlot::Number(_)) =>
							stack.push(StackSlot::Number(1.0)),
						(StackSlot::Number(_), StackSlot::String(_)) =>
							stack.push(StackSlot::Number(1.0)),
						_ => panic!("not equal is only supported for Strings and Numbers")
					}
				},
				Command::Not => {
					if let StackSlot::Number(n) = stack.pop().unwrap() {
						stack.push(StackSlot::Number(
							if n == 0.0 {
								1.0
							} else {
								0.0
							}
						));
					}
				},
				Command::Dup => {
					stack.push(stack.stack[stack.stack.len() - 1].clone());
				},
				Command::Swap => {
					let top = stack.pop().unwrap();
					let bot = stack.pop().unwrap();

					stack.push(top);
					stack.push(bot);
				},
				Command::Drop => {
					stack.pop();
				},
				Command::Print => {
					match stack.pop() {
						Some(slot) => match slot {
							StackSlot::Number(n) => print!("{}", n),
							StackSlot::String(s) => print!("{}", s.replace("\\n", "\n")),
							StackSlot::Reference(r, p) => print!("{} -> {}", r, p)
						},
						None => print!("Stack underflow!")
					};
				},
				Command::Put => {
					if let StackSlot::Reference(_, pos) = stack.pop().unwrap() {
						match stack.pop().unwrap() {
							StackSlot::Number(n) => program[pos + 1] = Command::Pushn(n),
							StackSlot::String(s) => program[pos + 1] = Command::Pushs(s),
							StackSlot::Reference(r, p) => program[pos + 1] = Command::Reference(String::from("@") + r.as_ref())
						};
					} else {
						panic!("reference required for put");
					}
				},
				Command::Push => {
					if let StackSlot::Reference(_, pos) = stack.pop().unwrap() {
						match stack.pop().unwrap() {
							StackSlot::Number(n) => program.insert(pos + 1, Command::Pushn(n)),
							StackSlot::String(s) => program.insert(pos + 1, Command::Pushs(s)),
							StackSlot::Reference(r, p) => program.insert(pos + 1, Command::Reference(String::from("@") + r.as_ref()))
						};
					} else {
						panic!("reference required for put");
					}
				}
				Command::Reference(s) => {
					let name: &str = s.split("@").collect::<Vec<&str>>()[1];
					if definitions.contains_key(name) {
						stack.push(StackSlot::Reference(String::from(name), definitions[name]));
					} else {
						panic!("no symbol \"{}\"", name);
					}
				}
				Command::AddressOf => {
					if let StackSlot::String(name) = stack.pop().unwrap() {
						let s: &str = name.as_ref();
						if definitions.contains_key(s) {
							stack.push(StackSlot::Reference(String::from(s), definitions[s]));
						} else {
							panic!("no symbol \"{}\"", name);
						}
					} else {
						panic!("string required");
					}
				}
				Command::PrintStack => println!("{:?}", stack.stack),
				_ => {}
			}
		}

		idx += 1;
	}
}

fn lexer(program: String) -> Vec<Command> {
	let mut commands: Vec<Command> = Vec::new();
	let mut idx = 0;
	let prog: Vec<&str> = program.split_whitespace().collect();

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
				"push" =>
					Command::Push,
				"+" =>
					Command::Add,
				"-" =>
					Command::Sub,
				"*" =>
					Command::Mul,
				"/" =>
					Command::Div,
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
				s if String::from(s).starts_with(r#"""#) => {
					let mut string = String::from(s);

					while !prog[idx].ends_with(r#"""#) {
						idx += 1;
						string.push(' ');
						string.push_str(prog[idx].as_ref());
					}

					Command::Pushs(String::from(string).replace(r#"""#, ""))
				},
				s if String::from(s).starts_with("@") => {
					if String::from(s).ends_with("!") {
						let mut name = String::from(s);
						name.pop();
						commands.push(Command::Reference(name));
						Command::Jmp
					} else {
						Command::Reference(String::from(s))
					}
				},
				s if s.parse::<f64>().is_ok() =>
					Command::Pushn(s.parse::<f64>().unwrap()),
				s =>
					Command::Pushs(String::from(s))
			};
		commands.push(next);
		idx += 1;
	}

	commands
}

fn main() {
	let args: Vec<String> = env::args().collect();

	let comment = Regex::new(r"/\*.*\*/").unwrap();

	let mut file =
		if args.len() > 1 {
			File::open(&args[1])
				.expect("file not found")
		}
		else {
			unsafe { File::from_raw_fd(0) }
		};

	let mut content = String::new();

	file.read_to_string(&mut content)
		.expect("unable to read file");

	let result = comment.replace_all(content.as_ref(), "");

	run(lexer(result.to_string()));
}
