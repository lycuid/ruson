use ruson::{
    cli::{Cli, CliFlag, CliOption},
    error::RusonResult,
    json::{
        formatter::{JsonFormat, JsonFormatter},
        query::JsonQuery,
        token::JsonProperty,
        JsonTokenLexer,
    },
};
use std::{
    collections::HashMap,
    io::{self, Read},
};

fn main() -> Result<(), String> {
    let app_name: String = std::env::args().next().unwrap();
    let internal_error = format!("{} internal error.", app_name);

    let mut rusoncli = Cli::new(&app_name);
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
            default: Some(format!("{}", JsonProperty::Root)),
            flag: CliFlag {
                short: "-q",
                long: Some("--query"),
                description: vec![
                    "Query for extracting desired 'json'".into(),
                    "subtree. The root 'json' tree must".into(),
                    format!("be referred as '{}'", JsonProperty::Root),
                ],
            },
        });

    let mut args = std::env::args().skip(1);
    let mut cliflags: Vec<String> = Vec::new();
    let mut clioptions: HashMap<&str, String> = HashMap::new();

    let json_filepath = rusoncli
        .parse_and_populate(&mut args, &mut cliflags, &mut clioptions)
        .unwrap_or_exit();

    for flag in cliflags.iter() {
        match flag.as_str() {
            "-h" => {
                println!("{}", rusoncli);
                std::process::exit(0)
            }
            "-v" => {
                println!("0.1.0");
                std::process::exit(0)
            }
            _ => {}
        }
    }

    let query_string = clioptions.get("query").ok_or(&internal_error)?;
    let json_query = JsonQuery::new(query_string).unwrap_or_exit_with(2);

    let json_string = if let Some(filepath) = json_filepath {
        std::fs::read_to_string(filepath.clone())
            .or_else(|err| Err(format!("'{}' {}", filepath, err)))
            .unwrap_or_exit()
    } else {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .or(Err("cannot read from stdin."))
            .unwrap_or_exit();
        buffer
    };

    let json_token = JsonTokenLexer::new(&json_string)
        .tokenize()
        .unwrap_or_exit()
        .apply(&json_query)
        .unwrap_or_exit();

    let mut json_formatter = JsonFormatter::new(&json_token);
    for flag in cliflags.iter() {
        match flag.as_str() {
            "-p" => {
                json_formatter.with(JsonFormat::Pretty);
                break;
            }
            "-t" => {
                json_formatter.with(JsonFormat::Table);
                break;
            }
            _ => {}
        }
    }

    println!("{}", json_formatter);

    Ok(())
}
