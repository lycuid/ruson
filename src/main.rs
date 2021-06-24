use ruson::{
    cli::{Cli, CliFlag, CliOption},
    error::RusonResult,
    json::{token::Jsonfmt, JsonParser},
    query::Query,
};
use std::{
    collections::HashMap,
    io::{self, Read},
};

fn main() -> Result<(), String> {
    let app_name: String = std::env::args().take(1).collect();
    let internal_error = format!("{} internal error.", app_name);

    let mut rusoncli = Cli::new(app_name.as_str());

    rusoncli.set_description(vec![
        "This script is used for serializing and extracting sub",
        "trees from json compliant syntax text.",
    ]);
    rusoncli.set_footer(vec![
        "For examples, refer to the manpage. For detailed",
        "documentation Visit: https://github.com/lycuid/ruson#readme",
    ]);

    rusoncli.add_option(CliOption {
        name: "query",
        default: Some(String::from("data")),
        flag: CliFlag {
            short: "-q",
            long: Some("--query"),
            exit_with_text: None,
            description: vec![
                "Query for extracting desired 'json'",
                "subtree. The root 'json' tree must",
                "be referred as 'data'",
            ],
        },
    });
    rusoncli.add_option(CliOption {
        name: "format",
        default: Some(String::from("raw")),
        flag: CliFlag {
            short: "-f",
            long: Some("--format"),
            exit_with_text: None,
            description: vec![
                "Output print format for 'json'.",
                "valid formats: (json|pretty|table)",
            ],
        },
    });

    let mut cliflags: Vec<String> = Vec::new();
    let mut clioptions: HashMap<&str, String> = HashMap::new();
    let json_filepath = rusoncli
        .parse(&mut cliflags, &mut clioptions)
        .or_exit_with(2);

    let json_string = if let Some(filepath) = json_filepath {
        std::fs::read_to_string(filepath.clone())
            .or_else(|err| Err(format!("'{}' {}", filepath, err)))
            .or_exit()
    } else {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .or(Err("cannot read from stdin."))
            .or_exit();
        buffer
    };

    let query_string = clioptions
        .get("query")
        .ok_or(internal_error.as_str())?
        .as_str();
    let json_query = Query::new(query_string).or_exit_with(128);

    // filesize 4.2 MiB: parsing 'jsontoken' ~ 155ms.
    let json_token = JsonParser::new(json_string.as_str())
        .parse()
        .or_exit()
        // filesize 4.2 MiB: applying 'query' on 'jsontoken' ~ 50ms.
        .apply(&json_query)
        .or_exit_with(128);

    // filesize 4.2 MiB: printing to stdout ~ 60ms.
    match clioptions
        .get("format")
        .ok_or(internal_error.as_str())?
        .to_ascii_lowercase()
        .as_str()
    {
        "raw" => println!("{}", Jsonfmt::Raw(&json_token)),
        "pretty" => println!("{}", Jsonfmt::Pretty(&json_token)),
        "table" => println!("{}", Jsonfmt::Table(&json_token)),
        _ => {
            Err::<(), &str>("Invalid '--format' value").or_exit();
        }
    }

    Ok(())
}
