include config.mk

build: fmt clean
	cargo build --release $(CARGO_FLAGS)

run:
	cargo run $(CARGO_FLAGS)

test: fmt clean
	cargo test $(CARGO_FLAGS)

doc: fmt clean
	cargo doc $(CARGO_FLAGS)

fmt:
	cargo fmt

clean:
	cargo clean
	rm -rf Cargo.lock

readme: src/lib.rs
	sed -En 's/\/\/\!\s?(.*)/\1/p' src/lib.rs > README.md

benchmark: config.mk
	./benchmark/run.sh $(BIN) -- $(BENCHMARK_URLS)

preinstall: config.mk
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

.PHONY: run fmt clean
