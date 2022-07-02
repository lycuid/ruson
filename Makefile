NAME=$(shell sed -En 's/name[^\"]+\"([^\"]+).*$$/\1/p' Cargo.toml)
VERSION=$(shell sed -En 's/version\ ?\=\ ?\"([^\"]+).*$$/\1/p' Cargo.toml)
AUTHOR=$(shell sed -En 's/author[^\"]+\"([^\"]+).*$$/\1/p' Cargo.toml)

BIN=target/release/$(NAME)
PREFIX=/usr/local
BINPREFIX=$(PREFIX)/bin
MANPREFIX=$(PREFIX)/share/man/man1

BENCHMARK_URLS=https://api.github.com/repos/rust-lang/rust          \
               https://api.github.com/repos/rust-lang/rust/commits  \
               https://api.github.com/repos/torvalds/linux          \
               https://api.github.com/repos/torvalds/linux/commits

build:
	cargo fmt
	cargo clean
	cargo build --release $(CARGO_FLAGS)

readme: src/lib.rs
	sed -En 's/\/\/\!\s?(.*)/\1/p' src/lib.rs | tee README.md | xclip -selection clipboard

.PHONY: benchmark
benchmark:
	./benchmark/run.sh $(BIN) -- $(BENCHMARK_URLS)

preinstall:
	mkdir -p $(DESTDIR)$(BINPREFIX)
	mkdir -p $(DESTDIR)$(MANPREFIX)

install: $(BIN) preinstall
	strip $(BIN)
	cp $(BIN) $(DESTDIR)$(BINPREFIX)/$(NAME)
	sed -e 's/APPNAME/$(NAME)/g' \
		-e 's/APPVERSION/$(VERSION)/g' \
		-e 's/APPAUTHOR/$(AUTHOR)/g' $(NAME).1 \
		| gzip > $(DESTDIR)$(MANPREFIX)/$(NAME).1.gz
	chmod 755 $(DESTDIR)$(BINPREFIX)/$(NAME)
	chmod 644 $(DESTDIR)$(MANPREFIX)/$(NAME).1.gz

uninstall:
	rm $(DESTDIR)$(BINPREFIX)/$(NAME)
	rm $(DESTDIR)$(MANPREFIX)/$(NAME).1.gz
