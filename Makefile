BUILDFLAGS=--release
CARGO=cargo $(CARGOFLAGS)
BIN=./target/release/ruson
BENCHMARK_URLS=https://api.github.com/repos/rust-lang/rust \
	https://api.github.com/repos/rust-lang/rust/commits

build: fmt
	$(CARGO) build $(BUILDFLAGS) && strip $(BIN)

fmt:
	$(CARGO) fmt

check:
	$(CARGO) check

clean:
	$(CARGO) clean

test: fmt src
	$(CARGO) test

benchmark: build
	$(SHELL) ./benchmark/run.sh $(BENCHMARK_URLS)

readme:
	sed -e '/{{benchmarks}}/ {' \
		-e 'r'<(make clean benchmark --quiet CARGOFLAGS='--quiet') \
		-e 'd' -e '}' README.tmpl > README

.PHONY: fmt clean benchmark
