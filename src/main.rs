use std::ffi::OsString;

use clap::{Arg, ArgAction, Command};
use tracing_subscriber::util::SubscriberInitExt;

use tan_toolchain::cmd::deps::handle_deps;
use tan_toolchain::cmd::format::handle_format;
use tan_toolchain::cmd::lint::handle_lint;
use tan_toolchain::cmd::repl::handle_repl;
use tan_toolchain::cmd::run::handle_run;
use tan_toolchain::cmd::test::handle_test;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .without_time()
        .with_target(false)
        .finish()
        .init();
}

fn main() -> anyhow::Result<()> {
    init_tracing();

    let run_cmd = Command::new("run")
        .about("Run a Tan program")
        .aliases(["r", "exec"])
        .arg(
            Arg::new("PATH")
                .help("The path of the program")
                .default_value("."), // if the path is missing default to the current directory
        )
        // #todo add --profile, --dev, --prod
        .arg(
            Arg::new("program_args")
                .help("Extra arguments to pass to the program")
                .action(ArgAction::Append)
                .last(true), // #todo consider .trailing_var_arg(true)
        );

    // #todo This is a temp name.
    // #todo Consider other names, like `crate`, `package`, `pm`, or even `cargo`.
    let deps_cmd = Command::new("deps")
        .about("Dependency management")
        .subcommand(
            Command::new("install")
                .about("Install the dependencies")
                .alias("i"),
        );

    let lint_cmd = Command::new("lint").about("Lint a Tan text file").arg(
        Arg::new("PATH")
            .help("The path of the text")
            .required(true)
            .index(1),
    );

    let format_cmd = Command::new("format")
        .about("Format a Tan text file")
        .alias("fmt")
        .arg(
            Arg::new("PATH")
                .help("The path of the text")
                .required(true)
                .index(1),
        );

    // #todo add more test parameters
    let test_cmd = Command::new("test")
        .about("Execute unit and integration tests")
        .arg(
            Arg::new("PATH")
                // #todo improve the help message
                .help("The base path for the tests")
                .default_value("."), // if the path is missing default to the current directory
        )
        .arg(
            Arg::new("FILE")
                .help("Filter the test files")
                // .short("f")
                .long("file")
                .required(false)
                .action(ArgAction::Set),
        )
        // #todo Not implemented yet.
        .arg(
            Arg::new("FUNCTION")
                .help("Filter the test functions")
                // .short("f")
                .long("function")
                .required(false)
                .action(ArgAction::Set),
        )
        .alias("t");

    let tan_cmd = Command::new("tan")
        .bin_name("tan")
        .author("George Moschovitis, gmosx@reizu.org")
        .version(VERSION)
        .about("A CLI for the Tan Language")
        .allow_external_subcommands(true)
        .subcommand(run_cmd)
        .subcommand(lint_cmd)
        .subcommand(format_cmd)
        .subcommand(deps_cmd)
        .subcommand(test_cmd);

    let matches = tan_cmd.get_matches();

    match matches.subcommand() {
        Some((subcommand, subcommand_matches)) => match subcommand {
            "run" => handle_run(subcommand_matches)?,
            "lint" => handle_lint(subcommand_matches)?,
            "format" => handle_format(subcommand_matches)?,
            "deps" => handle_deps(subcommand_matches)?,
            "test" => handle_test(subcommand_matches)?,
            _ => {
                // #todo extract this as a function.
                // Try to run an external subcommand (e.g. a tan plugin)

                let mut subcommand_args: Vec<_> = subcommand_matches
                    .get_many::<OsString>("")
                    .unwrap()
                    .collect();

                // Prepend the subcommand name to the arguments, to match the
                // corresponding Cargo plugin convention.
                let s = OsString::from(subcommand);
                subcommand_args.insert(0, &s);

                // Convert the subcommand name into a standardized tan plugin naming scheme.
                let subcommand = format!("tan-{subcommand}");

                // #todo better error reporting.
                let mut child = std::process::Command::new(subcommand)
                    .args(subcommand_args)
                    .spawn()
                    .expect("spawning the subcommand should not fail");

                let status = child.wait().expect("should wait on child");

                std::process::exit(status.code().unwrap_or_default());
            }
        },
        None => {
            // By default execute the repl command.
            handle_repl()?;
        }
    }

    Ok(())
}
