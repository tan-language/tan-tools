use clap::{Arg, ArgMatches, Command};
use rustyline::{error::ReadlineError, Editor};
use tan::util::format::format_pretty_error;
use tan::{
    eval::{env::Env, eval},
    lexer::Lexer,
    parser::Parser,
};
use tan_fmt::compact::format_compact;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const HISTORY_FILENAME: &str = ".tan_history.txt";

// #TODO properly implement this.
fn run(run_matches: &ArgMatches) -> anyhow::Result<()> {
    let path: &String = run_matches
        .get_one("PATH")
        .expect("missing path to program file");

    let input = std::fs::read_to_string(path).expect("cannot read input");

    let mut lexer = Lexer::new(&input);
    let tokens = lexer.lex().expect("cannot lex");
    dbg!(&tokens);

    let mut parser = Parser::new(tokens);
    let expr = parser.parse();
    dbg!(&expr);

    Ok(())
}

// #TODO rename to `shell` or something else?
fn repl() -> anyhow::Result<()> {
    // #TODO support completer!
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new()?;

    if rl.load_history(HISTORY_FILENAME).is_err() {
        println!("No previous history.");
    }

    println!("Tan, press CTRL-D to exit.");

    loop {
        // #TODO what would be a cool prompt?
        // #TODO try to use the legendary `READY` in some capacity.
        let readline = rl.readline("> ");

        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());

                let mut lexer = Lexer::new(&line);
                let result = lexer.lex();

                let Ok(tokens) = result else {
                    eprintln!("{}", format_pretty_error(&result.unwrap_err(), &line, None));
                    continue;
                };

                let mut parser = Parser::new(tokens);
                let result = parser.parse();

                let Ok(expr) = result else {
                    eprintln!("{}", format_pretty_error(&result.unwrap_err(), &line, None));
                    continue;
                };

                let result = eval(expr, &mut Env::default());

                let Ok(value) = result else {
                    println!("Eval error: {}", result.unwrap_err());
                    continue;
                };

                println!("{}", format_compact(&value));
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    rl.save_history(HISTORY_FILENAME).unwrap();

    Ok(())
}

fn main() -> anyhow::Result<()> {
    // #TODO consider a different name? even though 'run' is generic enough.
    let run_cmd = Command::new("run").about("Run a Tan program").arg(
        Arg::new("PATH")
            .help("The path of the program")
            .required(true)
            .index(1),
    );

    let tan_cmd = Command::new("tan")
        .bin_name("tan")
        .author("George Moschovitis, gmosx@reizu.org")
        .version(VERSION)
        .about("A CLI for Tan")
        .subcommand(run_cmd);

    let matches = tan_cmd.get_matches();

    if let Some(run_matches) = matches.subcommand_matches("run") {
        run(run_matches)?;
    } else {
        repl()?;
    }

    Ok(())
}
