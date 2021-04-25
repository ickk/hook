.PHONY: run build

CARGO:=~/.cargo/bin/cargo

run:
	$(CARGO) run --release

build:
	$(CARGO) build --release
	cp target/release/hook ./
