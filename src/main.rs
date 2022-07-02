use ruson::{
    cli::{Cli, CliFlag, CliOption},
    error::RusonResult,
    json::{
        formatter::{Formatter, PrettyJson, RawJson, TableJson},
        lexer::JsonLexer,
        query::JsonQuery,
        token::Json,
    },
};
use std::{
    collections::HashMap,
    io::{self, Read},
};

pub const NAME: &'static str = env!("CARGO_PKG_NAME");
pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() -> Result<(), String> {
    let rusoncli = create_cli(NAME);

    let mut args = std::env::args().skip(1);
    let mut cliflags: Vec<String> = Vec::new();
    let mut clioptions: HashMap<&str, String> = HashMap::new();
    let json_filepath = rusoncli
        .parse_and_populate(&mut args, &mut cliflags, &mut clioptions)
        .unwrap_or_exit_with(2);

    let mut json_formatter: Box<dyn Formatter<Token = Json>> =
        Box::new(RawJson {});

    for flag in cliflags.iter() {
        match flag.as_str() {
            "-p" => json_formatter = Box::new(PrettyJson { indent: "  " }),
            "-t" => json_formatter = Box::new(TableJson {}),
            "-v" => Err(format!(" {}", VERSION)).unwrap_or_exit_with(0),
            "-h" => {
                println!("{}", rusoncli);
                std::process::exit(0);
            }
            _ => continue,
        }
    }

    // construct query.
    let query_string = clioptions
        .get("query")
        .ok_or(format!(" internal error."))
        .unwrap_or_exit();
    let json_query = JsonQuery::new(query_string).unwrap_or_exit_with(2);

    // read json string from file or stdin.
    let json_string = if let Some(path) = json_filepath {
        std::fs::read_to_string(&path)
            .or_else(|err| Err(format!(" '{}' {}", path, err)))
    } else {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .and(Ok(buffer))
            .or(Err(" cannot read from stdin.".into()))
    }
    .unwrap_or_exit();

    // tokenize json string.
    let json_token = JsonLexer::new(&json_string)
        .tokenize()
        .unwrap_or_exit()
        .apply(&json_query)
        .unwrap_or_exit();

    Ok(println!("{}", json_formatter.dump(&json_token)))
}

#[inline(always)]
pub fn create_cli(name: &'static str) -> Cli {
    let mut cli = Cli::new(name);
    cli.set_description(vec![
        "Extract sub tree from valid 'json' text.".into(),
        "Use standard input, if FILE not provided.".into(),
    ])
    .set_footer(vec![
        "For examples, refer to the manpage. For detailed".into(),
        "documentation Visit: https://github.com/lycuid/ruson#readme".into(),
    ])
    .add_flag(CliFlag {
        short: "-p",
        long: Some("--pretty"),
        description: vec!["Print pretty formatted 'json'.".into()],
    })
    .add_flag(CliFlag {
        short: "-t",
        long: Some("--table"),
        description: vec!["Print table formatted 'json'.".into()],
    })
    .add_option(CliOption {
        name: "query",
        default: Some("".into()),
        flag: CliFlag {
            short: "-q",
            long: Some("--query"),
            description: vec![
                "Query for extracting desired 'json' subtree.".into()
            ],
        },
    });
    cli
}
