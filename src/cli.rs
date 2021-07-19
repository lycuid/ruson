//! Posix compliant command line arguemnt parser and processor.
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
}

impl<'a> CliFlag<'a> {
    /// exact match of either `short` or `long` argument.
    pub fn matches(&self, arg: &str) -> bool {
        [self.short, self.long.unwrap_or("")].contains(&arg)
    }
}

/// Command line Argument Options (always' accept arguments
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
    pub fn assoc_value(&self, arg: &str) -> Option<String> {
        let mut argparser = Parser::new(&arg);
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
    description: Lines,
    footer: Lines,
    /// using `Vec` instead of `HashMap` to preserve order.
    flags: Vec<CliFlag<'a>>,
    /// using `Vec` instead of `HashMap` to preserve order.
    options: Vec<CliOption<'a>>,
}

impl<'a> Cli<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            description: vec![],
            footer: vec![],
            flags: vec![
                CliFlag {
                    short: "-h",
                    long: Some("--help"),
                    description: vec!["Display this help and exit.".into()],
                },
                CliFlag {
                    short: "-v",
                    long: Some("--version"),
                    description: vec!["Display version and exit.".into()],
                },
            ],
            options: vec![],
        }
    }

    pub fn set_description(&mut self, description: Lines) -> &mut Self {
        self.description = description;
        self
    }

    pub fn set_footer(&mut self, footer: Lines) -> &mut Self {
        self.footer = footer;
        self
    }

    pub fn add_flag(&mut self, flag: CliFlag<'a>) -> &mut Self {
        self.flags.push(flag);
        self
    }

    pub fn add_option(&mut self, option: CliOption<'a>) -> &mut Self {
        self.options.push(option);
        self
    }

    fn empty_err(key: &str) -> String {
        format!("'{}' cannot be empty.", key)
    }

    /// parses and populates `Vec<flag.short>` and `HashMap<option.name, value>`.
    ///
    /// Returns:
    /// - `Err(String)`: argument parse error (malformed arguments etc).
    /// - `Ok(Some(filepath))`: no parse error, read from file.
    /// - `Ok(None)`: no parse error, read from stdin.
    pub fn parse_and_populate<I: Iterator<Item = String>>(
        &self,
        args: &mut I,
        flags: &mut Vec<String>,
        options: &mut std::collections::HashMap<&'a str, String>,
    ) -> Result<Option<String>, String> {
        // populating with options that have default value.
        for option in self.options.iter() {
            if let Some(value) = &option.default {
                options.insert(option.name, value.clone());
            }
        }

        'mainloop: while let Some(arg) = args.next() {
            let mut chars = arg.chars();

            match chars.next() {
                Some('-') => match chars.next() {
                    // read from stdin (single hyphen).
                    None => break,
                    Some('-') => {
                        // handle long options only (starts with double hyphen).
                        if chars.next().is_some() {
                            // try matching flags, continue mainloop if found.
                            for flag in self.flags.iter() {
                                if flag.matches(&arg) {
                                    flags.push(String::from(flag.short));
                                    continue 'mainloop;
                                }
                            }
                            // try matching options, continue mainloop if found.
                            for opt in self.options.iter() {
                                if opt.flag.matches(&arg) {
                                    args.next()
                                        .and_then(|next| {
                                            options.insert(opt.name, next);
                                            Some(())
                                        })
                                        .ok_or(Self::empty_err(opt.name))?;
                                    continue 'mainloop;
                                }
                                if let Some(value) = opt.assoc_value(&arg) {
                                    options.insert(opt.name, value);
                                    continue 'mainloop;
                                }
                            }
                        }
                        // end of command (just double hyphen).
                        return Ok(Some(arg));
                    }
                    // single hyphen followed by non hyphen character[s]:
                    // keep matching `flags`, when fails, try to match the
                    // immediate next char with `options`.
                    // if that fails, then return error (invalid flag).
                    // or take the rest of the string (if exists) as the value.
                    // if no rest, then move to next arguemnt as the value.
                    Some(ch) => {
                        let mut flag_arg = format!("-{}", ch);
                        // keep parsing flags, until it doesn't match
                        let maybe_option = 'flags: loop {
                            for flag in self.flags.iter() {
                                if flag.matches(&flag_arg) {
                                    flags.push(flag_arg);
                                    // try calling for the next flag from the
                                    // flag group.
                                    if let Some(next_ch) = chars.next() {
                                        flag_arg = format!("-{}", next_ch);
                                        continue 'flags;
                                    } else {
                                        break 'flags None;
                                    }
                                }
                            }
                            break Some(flag_arg); // consider this as option.
                        };

                        if let Some(opt) = maybe_option {
                            for option in self.options.iter() {
                                if option.flag.matches(&opt) {
                                    // trying to handle arguemnts like `-ovalue`
                                    // where `-o` is the argument and `value`
                                    // is the value.
                                    let rest: String = chars.collect();
                                    let value = if rest.is_empty() {
                                        args.next()
                                            .ok_or(Self::empty_err(
                                                option.name,
                                            ))?
                                            .to_owned()
                                    } else {
                                        rest
                                    };
                                    options.insert(option.name, value);
                                    continue 'mainloop;
                                }
                            }
                            // return `Err("Invalid flag")`, if doesn't match
                            // any flag or option.
                            return Err(format!("Invalid flag: '{}'.", opt));
                        }
                    }
                },
                // return arg as the 'default' argument.
                // if it doesn't start with a hyphen (`-`).
                _ => return Ok(Some(arg)),
            }
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
