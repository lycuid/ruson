use ruson::{
    cli::{Cli, CliFlag, CliOption},
    error::RusonResult,
    json::{
        query::JsonQuery,
        token::{JsonProperty, Jsonfmt},
        JsonTokenLexer,
    },
};
use std::{
    collections::HashMap,
    io::{self, Read},
};

fn main() -> Result<(), String> {
    let app_name: String = std::env::args().take(1).collect();
    let app_version = "v1.0.0";
    let internal_error = format!("{} internal error.", app_name);

    let mut rusoncli = Cli::new(&app_name, &app_version);

    rusoncli.set_description(vec![
        "Extract sub tree from valid 'json' text.".into()
    ]);
    rusoncli.set_footer(vec![
        "For examples, refer to the manpage. For detailed".into(),
        "documentation Visit: https://github.com/lycuid/ruson#readme".into(),
    ]);

    rusoncli.add_option(CliOption {
        name: "query",
        default: Some("root".into()),
        flag: CliFlag {
            short: "-q",
            long: Some("--query"),
            exit_with_text: None,
            description: vec![
                "Query for extracting desired 'json'".into(),
                "subtree. The root 'json' tree must".into(),
                format!("be referred as '{}'", JsonProperty::Data),
            ],
        },
    });
    rusoncli.add_option(CliOption {
        name: "format",
        default: Some("raw".into()),
        flag: CliFlag {
            short: "-f",
            long: Some("--format"),
            exit_with_text: None,
            description: vec![
                "Output format for parsed 'json'.".into(),
                "valid FORMAT: json, pretty, table".into(),
            ],
        },
    });

    let mut cliflags: Vec<String> = Vec::new();
    let mut clioptions: HashMap<&str, String> = HashMap::new();
    let json_filepath = rusoncli
        // this may exit with error code '0', when any flag with
        // `Some(exit_with_text)` gets parsed.
        .parse(&mut cliflags, &mut clioptions)
        .unwrap_or_exit();

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

    let query_string = clioptions.get("query").ok_or(&internal_error)?;
    let json_query = JsonQuery::new(query_string).unwrap_or_exit_with(2);

    // filesize 4.2 MiB: parsing 'jsontoken' ~ 155ms.
    let json_token = JsonTokenLexer::new(&json_string)
        .tokenize()
        .unwrap_or_exit()
        // filesize 4.2 MiB: applying 'query' on 'jsontoken' and returning
        // the cloned subtree 'jsontoken' ~ 50ms.
        .apply(&json_query)
        .unwrap_or_exit();

    // filesize 4.2 MiB: printing to stdout ~ 60ms.
    match clioptions
        .get("format")
        .ok_or(&internal_error)?
        .to_ascii_lowercase()
        .as_str()
    {
        "raw" => println!("{}", Jsonfmt::Raw(&json_token)),
        "pretty" => println!("{}", Jsonfmt::Pretty(&json_token)),
        "table" => println!("{}", Jsonfmt::Table(&json_token)),
        _ => {
            Err::<(), &str>("Invalid '--format' value").unwrap_or_exit_with(2);
        }
    }

    Ok(())
}
