use ruson::{
    cli::{Cli, CliFlag, CliOption},
    error::RusonResult,
    json::{
        formatter::{PrettyJson, RawJson, TableJson},
        query::JsonQuery,
        token::JsonToken,
        JsonTokenLexer,
    },
    utils::Formatter,
};
use std::{
    collections::HashMap,
    io::{self, Read},
};

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() -> Result<(), String> {
    let mut rusoncli = Cli::new(NAME);
    rusoncli
        .set_description(vec![
            "Extract sub tree from valid 'json' text.".into(),
            "Use standard input, if FILE not provided.".into(),
        ])
        .set_footer(vec![
            "For examples, refer to the manpage. For detailed".into(),
            "documentation Visit: https://github.com/lycuid/ruson#readme"
                .into(),
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
                    "Query for extracting desired 'json' subtree.".into(),
                ],
            },
        });

    let mut args = std::env::args().skip(1);
    let mut cliflags: Vec<String> = Vec::new();
    let mut clioptions: HashMap<&str, String> = HashMap::new();

    let json_filepath = rusoncli.parse_and_populate(
        &mut args,
        &mut cliflags,
        &mut clioptions,
    )?;

    let mut json_formatter: Box<dyn Formatter<Token = JsonToken>> =
        Box::new(RawJson {});

    for flag in cliflags.iter() {
        match flag.as_str() {
            "-p" => json_formatter = Box::new(PrettyJson { padding: "  " }),
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
        .ok_or(format!("{} internal error.", NAME))?;
    let json_query = JsonQuery::new(query_string).unwrap_or_exit_with(2);

    // tokenize json string.
    let json_string = if let Some(filepath) = json_filepath {
        std::fs::read_to_string(&filepath)
            .or_else(|err| Err(format!("'{}' {}", filepath, err)))?
    } else {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .or(Err("cannot read from stdin."))?;
        buffer
    };
    let json_token = JsonTokenLexer::new(&json_string)
        .tokenize()
        .unwrap_or_exit()
        .apply(&json_query)?;

    println!("{}", json_formatter.dump(&json_token));

    Ok(())
}
