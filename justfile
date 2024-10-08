set positional-arguments

default: build

alias i := install
alias b := build
alias l := clippy
alias t := test
alias tv := test-verbose

install: build
    command cp -f target/release/todor ~/.local/bin/
    cd ~/.local/bin; ln -sf todor today
    cd ~/.local/bin; ln -sf todor tomorrow

build:
    cargo build --release

clippy: build
    cargo clippy

test: build
    cargo test

test-verbose: build
    cargo test -- --nocapture

gitmain:
    git checkout main
    git pull

@gh-start br:
    git checkout main
    git pull
    git branch $1
    git checkout $1

@gh-close:
    git checkout main
    git pull

@gh-push br:
    git push origin $1

@gh-release ver:
    git checkout main
    git tag $1
    git push --tags
    git push origin main
