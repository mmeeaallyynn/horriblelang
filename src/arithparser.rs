use crate::*;


pub fn parse(prog: &Vec<&str>, mut i: &mut usize) -> Vec<Command> {
    let parsed = parse_expression(prog, &mut i);

    *i -= 1;
    parsed
}

fn parse_expression(prog: &Vec<&str>, mut i: &mut usize) -> Vec<Command> {
    let mut parsed: Vec<Command> = vec![];

    match prog[*i] {
        "(" => parsed.append(&mut parse_binary_expression(prog, &mut i)),
        _ => parsed.append(&mut parse_value(prog, &mut i)),
    }

    parsed
}

fn parse_binary_expression(prog: &Vec<&str>, mut i: &mut usize) -> Vec<Command> {
    let mut parsed: Vec<Command> = vec![];

    if !(prog[*i] == "(") {
        error(&format!("expected opening parenthesis, found {}", prog[*i]));
        panic!();
    }
    *i += 1;

    parsed.append(&mut parse_expression(prog, &mut i));

    let op = match prog[*i] {
        "+" => Command::Add,
        "-" => Command::Sub,
        "*" => Command::Mul,
        ">" => Command::GT,
        ">=" => Command::GE,
        "<" => Command::LT,
        "<=" => Command::LE,
        "==" => Command::EQ,
        "!=" => Command::NE,
        _ => {
            error(&format!("invalid operator: {}", prog[*i]));
            panic!();
        }
    };
    *i += 1;

    parsed.append(&mut parse_expression(prog, &mut i));
    parsed.push(op);

    if !(prog[*i] == ")") {
//        error(&format!("expected closing parenthesis, found {}", prog[*i]));
//        panic!();
        parsed.append(&mut parse_expression_continuation(prog, &mut i));
    }
    *i += 1;

    parsed
}

fn parse_expression_continuation(prog: &Vec<&str>, mut i: &mut usize) -> Vec<Command> {
    warn("Notherlang: operator precedence is not supported, use parentheses instead!");

    let mut parsed: Vec<Command> = vec![];

    let op = match prog[*i] {
        "+" => Command::Add,
        "-" => Command::Sub,
        "*" => Command::Mul,
        ">" => Command::GT,
        ">=" => Command::GE,
        "<" => Command::LT,
        "<=" => Command::LE,
        "==" => Command::EQ,
        "!=" => Command::NE,
        _ => {
            error(&format!("invalid operator: {}", prog[*i]));
            panic!();
        }
    };
    *i += 1;

    parsed.append(&mut parse_expression(prog, &mut i));
    parsed.push(op);

    if !(prog[*i] == ")") {
        parsed.append(&mut parse_expression_continuation(prog, &mut i));
    }

    parsed
}

fn parse_value(prog: &Vec<&str>, i: &mut usize) -> Vec<Command> {
    let mut parsed: Vec<Command> = vec![];

    match prog[*i] {
        n if n.parse::<f64>().is_ok() => {
            parsed.push(Command::Pushn(prog[*i].parse::<f64>().unwrap()));
            *i += 1;
        },
        s if s.starts_with("@") && s.ends_with("!") => {
            let mut name = String::from(s);
            name.pop();
            parsed.push(Command::Reference(name));
            parsed.push(Command::Jmp);
            *i += 1;
        },
        s if s.starts_with("@") => {
            parsed.push(Command::Reference(s.into()));
            *i += 1;
        }
        s => {
            log("is a string!");
            parsed.push(Command::Pushs(s.into()));
            *i += 1;
        }
    }

    parsed
}

