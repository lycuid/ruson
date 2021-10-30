# RUSON
Command line json text parsing and processing utility.
parsing json compliant with [`rfc8259`](https://datatracker.ietf.org/doc/html/rfc8259).
source code _does not_ contain any third party dependencies (why? single source of truth).

- [Installation](#installation)
  * [from source](#from-source)
- [Usage](#usage)
  * [Synopsis](#synopsis)
  * [Arguments](#arguments)
- [Query Syntax](#query-syntax)
- [Examples](#examples)
- [Licence](#licence)

# INSTALLATION
### From source.
Requirements:
- [rust and cargo](https://www.rust-lang.org/)
- [gnu make](https://www.gnu.org/software/make/)
```sh
git clone --depth=1 https://github.com/lycuid/ruson.git
cd ruson
make && sudo make install
```
# USAGE
### SYNOPSIS
```txt
ruson [OPTIONS]... FILE
```
`FILE` can be replaced with `-` (hyphen) or skipped entirely to read json text from standard input.

### ARGUMENTS
`-p`, `--pretty`

Print pretty formatted 'json'.

`-t`, `--table`

Print table formatted 'json'.

`-q query`, `--query[=query]`

Text for extracting desired _**json**_ subtree.
_**query**_ text can be any valid javascript syntax of object property accessors or array indexing.

# Query Syntax.
Dot notation.
```sh
echo '{ "prop": "value" }' | ruson --query '.prop' # "value"
```

Bracket notation.
```sh
echo '{ "prop": "value" }' | ruson --query '["prop"]' # "value"
```

Array indexing.
```sh
echo '{ "prop": [1, 2, 3, 4, 5] }' | ruson --query '.prop[2]' # 3
```

`.keys()` Function.
```sh
echo '{ "one": 1, "two": 2, "three": 3 }' | ruson -q '.keys()' # ["one", "two", "three"]
```

`.values()` function.
```sh
echo '{ "one": 1, "two": 2, "three": 3 }' | ruson -q '.values()' # [1, 2, 3]
```

`.length()` function.
```sh
echo '[1, 2, 3]' | ruson -q '.length()' # 3
```

`.map()` function.
```sh
echo '{ "list": [{ "id": 1 }, { "id": 2 }, { "id": 3 }] }' | ruson -q'.list.map(.id)' # [1, 2, 3]
```

# EXAMPLES
Download latest `xkcd` comic
```sh
curl https://xkcd.com/info.0.json | ruson -q ".img" | xargs wget
```
Pokemon attack names.
```sh
curl https://pokeapi.co/api/v2/pokemon/pikachu | ruson -q ".moves[0].move.name"
```

# LICENCE
[GPLv3](https://www.gnu.org/licenses/gpl-3.0.en.html)
