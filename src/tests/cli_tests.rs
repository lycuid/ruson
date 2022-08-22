use crate::cli::*;
use std::collections::HashMap;

fn create_cli(name: &'static str) -> Cli {
    let mut cli = Cli::new(name);
    cli.add_flag(CliFlag {
        short: "-h",
        long: Some("--help"),
        description: vec![],
    })
    .add_flag(CliFlag {
        short: "-v",
        long: Some("--version"),
        description: vec![],
    })
    .add_flag(CliFlag {
        short: "-a",
        long: Some("--argument"),
        description: vec![],
    })
    .add_option(CliOption {
        name: "option1",
        default: Some("default".into()),
        flag: CliFlag {
            short: "-1",
            long: Some("--option1"),
            description: vec![],
        },
    })
    .add_option(CliOption {
        name: "option2",
        default: None,
        flag: CliFlag {
            short: "-2",
            long: Some("--option2"),
            description: vec![],
        },
    })
    .add_option(CliOption {
        name: "option3",
        default: None,
        flag: CliFlag {
            short: "-3",
            long: Some("--option3"),
            description: vec![],
        },
    })
    .add_option(CliOption {
        name: "option4",
        default: None,
        flag: CliFlag {
            short: "-4",
            long: Some("--option4"),
            description: vec![],
        },
    })
    .add_option(CliOption {
        name: "option5",
        default: Some("default".into()),
        flag: CliFlag {
            short: "-5",
            long: Some("--option5"),
            description: vec![],
        },
    });
    cli
}

#[test]
fn success_cli() {
    let cli = create_cli(env!("CARGO_PKG_NAME"));

    let mut flags: Vec<String> = vec![];
    let mut options: HashMap<&str, String> = HashMap::new();

    let mut args = vec![
        "-av1".into(),
        "value".into(),
        "-h2value".into(),
        "--option3".into(),
        "value".into(),
        "--option4=value".into(),
    ]
    .into_iter();

    let parsed = cli.parse_and_populate(&mut args, &mut flags, &mut options);
    assert!(parsed.is_ok(), "{:?}", parsed);

    assert_eq!(flags.len(), 3);
    for flag in flags.iter() {
        match flag.as_str() {
            "-h" | "-v" | "-a" => {}
            _ => panic!("Invalid flag: '{}'", flag),
        }
    }

    assert_eq!(options.len(), 5);
    for (key, value) in options.iter() {
        match key {
            &"option1" | &"option2" | &"option3" | &"option4" => {
                assert_eq!(*value, String::from("value"))
            }
            &"option5" => assert_eq!(*value, String::from("default")),
            _ => panic!("Invalid option: '{}'", key),
        }
    }
}
