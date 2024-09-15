default: build

alias i := install
alias b := build

install: build
    command cp -f target/release/todor ~/.local/bin/
    cd ~/.local/bin; ln -sf todor today
    cd ~/.local/bin; ln -sf todor tomorrow

build:
    cargo build --release

