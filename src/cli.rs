use std::collections::HashMap;

type Lines<'a> = Vec<&'a str>;

/// Flags dont accept arguments.
#[derive(Debug, Clone)]
pub struct CliFlag<'a> {
    pub short: &'a str,
    pub long: Option<&'a str>,
    /// flags which are unrelated to the program behaviour.
    /// As soon as this flag is parsed, the program will print
    /// this text and exit (with exit code 0).
    /// examples: --help, --version.
    pub exit_with_text: Option<String>,
    /// lines of string, to have smooth display,
    /// with probably similar width lines.
    pub description: Lines<'a>,
}

/// Options 'always' accept arguments.
/// unless the flag contains 'Some(exit_with_text)' value.
#[derive(Debug, Clone)]
pub struct CliOption<'a> {
    /// Display name for word argument in the Program Usage string.
    /// example: -f, --file <name>
    pub name: &'a str,
    /// default value for the current option.
    pub default: Option<String>,
    pub flag: CliFlag<'a>,
}

#[derive(Debug, Clone)]
pub struct Cli<'a> {
    pub name: &'a str,
    pub description: Lines<'a>,
    pub footer: Lines<'a>,
    pub flags: Vec<CliFlag<'a>>,
    pub options: Vec<CliOption<'a>>,
}

impl<'a> Cli<'a> {
    pub fn new(name: &'a str) -> Self {
        let default_flags = vec![];
        let default_options = vec![];

        Self {
            name,
            description: vec![],
            footer: vec![],
            flags: default_flags,
            options: default_options,
        }
    }

    pub fn set_description(&mut self, description: Lines<'a>) {
        self.description = description;
    }

    pub fn set_footer(&mut self, footer: Lines<'a>) {
        self.footer = footer;
    }

    pub fn add_flag(&mut self, flag: CliFlag<'a>) {
        self.flags.push(flag);
    }

    pub fn add_option(&mut self, option: CliOption<'a>) {
        self.options.push(option);
    }

    fn clisetup(&mut self) {
        self.add_flag(CliFlag {
            short: "-h",
            long: Some("--help"),
            exit_with_text: None,
            description: vec!["Display this help and exit."],
        });

        self.flags.last_mut().unwrap().exit_with_text =
            Some(format!("{}", self.clone()));
    }

    fn empty_query_message(&self, name: &str) -> String {
        vec![
            format!("'{}' cannot be empty.", name),
            format!("Try '{} --help' for more information.", self.name),
        ]
        .join("\n")
    }

    pub fn parse(
        &mut self,
        flags: &mut Vec<String>,
        options: &mut HashMap<&'a str, String>,
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

        'mainloop: loop {
            if let Some(arg) = args.next() {
                // check if matches any provided flags, continue loop if true.
                for flag in self.flags.iter() {
                    if [flag.short, flag.long.unwrap_or(flag.short)]
                        .contains(&arg.as_str())
                    {
                        if let Some(text) = &flag.exit_with_text {
                            println!("{}", text);
                            std::process::exit(0);
                        }

                        flags.push(arg);
                        continue 'mainloop;
                    }
                }

                // check if matches any provided options, continue loop if true.
                for CliOption { flag, name, .. } in self.options.iter() {
                    if [flag.short, flag.long.unwrap_or(flag.short)]
                        .contains(&arg.as_str())
                    {
                        args.next()
                            .and_then(|next| {
                                options.insert(name, next).and(Some(()))
                            })
                            .ok_or(self.empty_query_message(name))?;

                        continue 'mainloop;
                    }
                }

                // if arg present, but doesn't match any provided 'flags' or
                // 'options', then assume its the default argument.
                break Ok(Some(arg));
            }

            // if no 'arg' then return empty string as default argument.
            break Ok(None);
        }
    }
}

impl<'a> std::fmt::Display for Cli<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "USAGE:")?;
        writeln!(f, "        {} [FLAGS|OPTIONS]... <FILE>", self.name)?;
        writeln!(f, "        {} [FLAGS|OPTIONS]...", self.name)?;

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

                let printable_flag_description = flag
                    .description
                    .iter()
                    .map(|s| format!("\t\t{}\n", s))
                    .collect::<String>();
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

                let printable_option_description = opt
                    .flag
                    .description
                    .iter()
                    .map(|s| format!("\t\t{}\n", s))
                    .collect::<String>();
                write!(f, "{}", printable_option_description)?;
            }
            writeln!(f, "")?; // padding.
        }

        write!(f, "{}", self.footer.join("\n"))
    }
}
