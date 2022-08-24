use std::env;
use std::process;

use quizanalyze::Config;

fn help_message(program: &str) -> String
{
    let mut message = String::from("usage: ");
    message.push_str(program);
    message.push_str(" -s \"search string\" [args] /path/to/file");
    message.push_str("\n\nargs:");
    message.push_str("\n    -s    question to search");
    message.push_str("\n    -r, --rank    ranks all the questions by mapping");
    message.push_str("\n    -u, --unique    the question is an uid");
    message.push_str("\n    -e, --exact    only include exact matches");
    message.push_str("\n    -m    map choices to numbers (<split character>choice<split character>number)");

    message
}

fn main()
{
    let config = Config::build(env::args().skip(1)).unwrap_or_else(|err|
    {
        eprintln!("error parsing args: {err}");

        eprintln!("{}", help_message(&env::args().nth(0)
            .expect("first program argument should always exist")));

        process::exit(1);
    });

    if let Err(err) = quizanalyze::run(&config)
    {
        eprintln!("application error: {err}");
        process::exit(2);
    };
}