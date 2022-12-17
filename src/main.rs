use std::env;
use std::fs;

use rustyline::error::ReadlineError;
use rustyline::Editor;


fn main() {
    let mut env = horrible::Environment::new(vec![], vec![]);
    let args: Vec<String> = env::args().collect();

    let arg_string = args.iter().map(|i| { format!("\"{}\"", i) }).collect::<Vec<String>>().join(" ");
    horrible::run_string(&mut env, &format!("| {}", arg_string))
        .expect("unable to parse arguments");

    let mut rl = Editor::<()>::new();

    if args.len() > 1 {
        let mut filename = &args[1];

        if filename == "-i" {
            filename = &args[2];
        }

        let contents = fs::read_to_string(filename)
            .expect("Something went wrong reading the file");

        match horrible::run_string(&mut env, &contents) {
            Ok(_env) => {},
            Err(err) => println!("{}", err),
        };
    }
    else {
        loop {
            let readline = rl.readline(
                if env.execute  {
                    ">> "
                }
                else {
                    ".. "
                }
            );
            match readline {
                Ok(line) => {
                    rl.add_history_entry(line.as_str());
                    match horrible::run_string(&mut env, &line) {
                        Ok(_env) => {},
                        Err(err) => println!("{}", err),
                    };
                },
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break
                },
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break
                },
                Err(err) => {
                    println!("Error: {:?}", err);
                    break
                }
            }
        }
    }
}
