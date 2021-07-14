use super::parser::Parser;

type Lines = Vec<String>;

/// Command line Argument Flag (doesn't accept arguments).
#[derive(Debug, Clone)]
pub struct CliFlag<'a> {
    pub short: &'a str,
    pub long: Option<&'a str>,
    /// lines of string, to have smooth display,
    /// with probably similar width lines.
    pub description: Lines,
    /// flags which are unrelated to the program behaviour.
    /// As soon as this flag is parsed, the program will print
    /// this text and exit (with exit code 0).
    /// examples: --help, --version.
    pub exit_with_text: Option<String>,
}

impl<'a> CliFlag<'a> {
    pub fn matches(&self, match_string: &str) -> bool {
        [self.short, self.long.unwrap_or(self.short)].contains(&match_string)
    }

    pub fn handle_exit_with_text(&self) {
        if let Some(text) = &self.exit_with_text {
            println!("{}", text);
            std::process::exit(0);
        }
    }
}

/// Command line Argument Options (always' accept arguments
/// unless the flag contains `Some(exit_with_text)` value).
#[derive(Debug, Clone)]
pub struct CliOption<'a> {
    /// Display name for word argument in the Program Usage string.
    /// example: -f, --file <name>
    pub name: &'a str,
    /// default value for the current option.
    pub default: Option<String>,
    pub flag: CliFlag<'a>,
}

impl<'a> CliOption<'a> {
    /// parse long option with syntax `--option=value` and return `value`.
    pub fn get_assoc_value(&self, arg: &str) -> Option<String> {
        let mut argparser = Parser::new(arg);

        self.flag
            .long
            .and_then(|long| argparser.match_string(long))
            .and_then(|_| argparser.match_char('='))
            .or(None)
            .and_then(|_| {
                Some(argparser.stack[argparser.pointer..].iter().collect())
            })
    }
}

#[derive(Debug, Clone)]
pub struct Cli<'a> {
    name: &'a str,
    version: &'a str,
    description: Lines,
    footer: Lines,
    flags: Vec<CliFlag<'a>>,
    options: Vec<CliOption<'a>>,
}

impl<'a> Cli<'a> {
    pub fn new(name: &'a str, version: &'a str) -> Self {
        Self {
            name,
            version,
            description: vec![],
            footer: vec![],
            flags: vec![
                CliFlag {
                    short: "-h",
                    long: Some("--help"),
                    exit_with_text: None,
                    description: vec!["Display this help and exit.".into()],
                },
                CliFlag {
                    short: "-v",
                    long: Some("--version"),
                    exit_with_text: Some(format!("{} {}", name, version)),
                    description: vec!["Display version and exit.".into()],
                },
            ],
            options: vec![],
        }
    }

    pub fn set_description(&mut self, description: Lines) {
        self.description = description;
    }

    pub fn set_footer(&mut self, footer: Lines) {
        self.footer = footer;
    }

    pub fn add_flag(&mut self, flag: CliFlag<'a>) {
        self.flags.push(flag);
    }

    pub fn add_option(&mut self, option: CliOption<'a>) {
        self.options.push(option);
    }

    fn clisetup(&mut self) {
        let usage = format!("{}", self);
        // adding usage info as `Some(exit_with_text)` for the 'help' flag.
        self.flags.first_mut().unwrap().exit_with_text = Some(usage);
    }

    /// This may exit when any flag with `Some(exit_with_text)` gets parsed.
    pub fn parse(
        &mut self,
        flags: &mut Vec<String>,
        options: &mut std::collections::HashMap<&'a str, String>,
    ) -> Result<Option<String>, String> {
        let mut args = std::env::args().skip(1);

        self.clisetup();

        // adding the options to the return options hashmap.
        for option in self.options.iter() {
            options.insert(
                option.name,
                option.default.clone().unwrap_or(String::new()),
            );
        }

        'mainloop: while let Some(arg) = args.next() {
            // check if matches any provided flags, continue loop if true.
            for flag in self.flags.iter() {
                if flag.matches(&arg) {
                    // exit if flag has `Some(exit_with_text)`.
                    flag.handle_exit_with_text();

                    flags.push(arg);
                    continue 'mainloop;
                }
            }

            // check if matches any provided options, continue loop if true.
            for option in self.options.iter() {
                if option.flag.matches(&arg) {
                    args.next()
                        .and_then(|next| {
                            options.insert(option.name, next).and(Some(()))
                        })
                        .ok_or(format!("'{}' cannot be empty.", option.name))?;
                    continue 'mainloop;
                }

                if let Some(value) = option.get_assoc_value(&arg) {
                    options.insert(option.name, value);
                    continue 'mainloop;
                }
            }

            // if arg present, but doesn't match any provided 'flags' or
            // 'options', then assume its the default argument.
            return Ok(Some(arg));
        }

        Ok(None)
    }
}

impl<'a> std::fmt::Display for Cli<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "USAGE: {} [FLAGS|OPTIONS]... FILE", self.name)?;

        if !self.description.is_empty() {
            writeln!(f, "{}", self.description.join("\n"))?;
            writeln!(f, "")?; // padding.
        }

        if !self.flags.is_empty() {
            writeln!(f, "FLAGS:")?;
            for flag in self.flags.iter() {
                write!(f, "  {}", flag.short)?;
                if let Some(long_opt) = flag.long {
                    write!(f, ", {}", long_opt)?;
                }
                writeln!(f, "")?;

                let printable_flag_description: String = flag
                    .description
                    .iter()
                    .map(|s| format!("\t\t{}\n", s))
                    .collect();
                write!(f, "{}", printable_flag_description)?;
            }
            writeln!(f, "")?; // padding.
        }

        if !self.options.is_empty() {
            writeln!(f, "OPTIONS:")?;
            for opt in self.options.iter() {
                write!(f, "  {}", opt.flag.short)?;
                if let Some(long_opt) = opt.flag.long {
                    write!(f, ", {}", long_opt)?;
                }
                writeln!(f, " <{}>", opt.name)?;

                let printable_option_description: String = opt
                    .flag
                    .description
                    .iter()
                    .map(|s| format!("\t\t{}\n", s))
                    .collect();
                write!(f, "{}", printable_option_description)?;
            }
            writeln!(f, "")?; // padding.
        }

        write!(f, "{}", self.footer.join("\n"))
    }
}
