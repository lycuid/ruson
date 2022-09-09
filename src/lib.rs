//! # RUSON
//! Command line json text parsing and processing utility. parsing json compliant with [`rfc8259`](https://datatracker.ietf.org/doc/html/rfc8259).
//! Benchmarked faster than [`jq`](https://stedolan.github.io/jq/) (more than _**twice**_ as fast).
//! _No third party dependencies_.
//!
//! - [Installation](#installation)
//!   * [Requirements](#requirements)
//!   * [Build and Install](#build-and-install)
//! - [Usage](#usage)
//! - [Query Syntax](#query-syntax)
//! - [Examples](#examples)
//! - [Licence](#licence)
//!
//! # INSTALLATION
//! ### Requirements:
//! - [rust and cargo](https://www.rust-lang.org/)
//! - [GNU Make](https://www.gnu.org/software/make/)
//! - pkg-config (optional).
//! ### Build and Install.
//! ```sh
//! git clone --depth=1 https://github.com/lycuid/ruson.git
//! cd ruson
//! make && sudo make install
//! ```
//! # USAGE
//! ```txt
//! USAGE: ruson [FLAGS|OPTIONS]... FILE
//! Extract sub tree from valid 'json' text.
//! Use standard input, if FILE not provided.
//!
//! FLAGS:
//!   -h, --help
//!                 Display this help and exit.
//!   -v, --version
//!                 Display version and exit.
//!   -p, --pretty
//!                 Print pretty formatted 'json'.
//!   -t, --table
//!                 Print table formatted 'json'.
//!
//! OPTIONS:
//!   -q, --query <query>
//!                 Query for extracting desired 'json' subtree.
//! ```
//!
//! # Query Syntax.
//! ```sh
//! # Dot notation.
//! echo '{ "prop": "value" }' | ruson --query '.prop' # "value"
//!
//! # Bracket notation.
//! echo '{ "prop": "value" }' | ruson --query '["prop"]' # "value"
//!
//! # Array indexing.
//! echo '{ "prop": [1, 2, 3, 4, 5] }' | ruson --query '.prop[2]' # 3
//!
//! # '.keys()' function (valid for 'object').
//! echo '{ "one": 1, "two": 2, "three": 3 }' | ruson -q '.keys()' # ["one", "two", "three"]
//!
//! # '.values()' function (valid for 'object').
//! echo '{ "one": 1, "two": 2, "three": 3 }' | ruson -q '.values()' # [1, 2, 3]
//!
//! # '.length()' function (valid for 'array' and 'string').
//! echo '[1, 2, 3]' | ruson -q '.length()' # 3
//!
//! # '.map()' function (valid for 'array').
//! echo '{ "list": [{ "id": 1 }, { "id": 2 }, { "id": 3 }] }' | ruson -q'.list.map(.id)' # [1, 2, 3]
//! ```
//!
//! # EXAMPLES
//! Download latest _**xkcd**_ comic
//! ```sh
//! curl https://xkcd.com/info.0.json 2>/dev/null | ruson -q ".img" | xargs wget
//! ```
//! Pokemon attack names.
//! ```sh
//! curl https://pokeapi.co/api/v2/pokemon/pikachu 2>/dev/null | ruson -q ".moves[0].move.name"
//! ```
//!
//! # LICENCE
//! [GPLv3](https://www.gnu.org/licenses/gpl-3.0.en.html)
pub mod cli;
pub mod error;
pub mod json;
pub mod lexer;

#[cfg(test)]
mod tests;
