NAME=$(shell sed -En 's/name[^\"]+\"([^\"]+).*$$/\1/p' Cargo.toml)
VERSION=$(shell sed -En 's/version\ ?\=\ ?\"([^\"]+).*$$/\1/p' Cargo.toml)
AUTHOR=$(shell sed -En 's/author[^\"]+\"([^\"]+).*$$/\1/p' Cargo.toml)

BIN=target/release/$(NAME)
PREFIX=/usr/local
BINPREFIX=$(PREFIX)/bin
MANPREFIX=$(PREFIX)/share/man/man1

BENCHMARK_URLS=\
							https://api.github.com/repos/rust-lang/rust \
							https://api.github.com/repos/rust-lang/rust/commits
