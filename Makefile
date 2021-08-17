include config.mk

build: fmt clean
	cargo build --release

test: fmt clean
	cargo test

doc: fmt clean
	cargo doc

fmt:
	cargo fmt

clean:
	cargo clean

readme: src/lib.rs
	sed -En 's/\/\/\!\s?(.*)/\1/p' src/lib.rs | tee README.md

benchmark: build config.mk
	$(SHELL) ./benchmark/run.sh $(BENCHMARK_URLS)

install: $(BIN)
	strip $(BIN)
	mkdir -p $(DESTDIR)$(BINPREFIX)
	cp $(BIN) $(DESTDIR)$(BINPREFIX)/$(NAME)
	chmod 755 $(DESTDIR)$(BINPREFIX)/$(NAME)
	mkdir -p $(DESTDIR)$(MANPREFIX)
	sed -e 's/APPNAME/$(NAME)/g' \
		-e 's/APPVERSION/$(VERSION)/g' \
		-e 's/APPAUTHOR/$(AUTHOR)/g' $(NAME).1 \
		| gzip > $(DESTDIR)$(MANPREFIX)/$(NAME).1.gz
	chmod 644 $(DESTDIR)$(MANPREFIX)/$(NAME).1.gz

uninstall:
	rm $(DESTDIR)$(BINPREFIX)/$(NAME)
	rm $(DESTDIR)$(MANPREFIX)/$(NAME).1.gz
