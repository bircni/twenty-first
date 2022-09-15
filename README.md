# twenty-first

![GitHub CI](https://github.com/Neptune-Crypto/twenty-first/actions/workflows/main.yml/badge.svg)
![crates.io](https://img.shields.io/crates/v/twenty-first.svg)

A collection of cryptography functions written in Rust.

## Setup

### Ubuntu
 - curl -- `sudo apt install curl`
 - rustup -- `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` (installs `rustup`, `cargo`, `rustc` etc.)
 - source the rust environment `source $HOME/.cargo/env`
 - gnuplot -- `sudo apt install gnuplot`
 - build-essential (for `make`) -- `sudo apt install build-essential`
 - clone this repository -- `git clone ssh://git@neptune.builders:2222/core-team/twenty-first.git`. Last time I checked, the RSA key fingerprint was `SHA256:wQ9euDKumP5H8MY1J8F07IEIb6Qz9isGkaFY8uL6U/Y`
 - install `vscode`
 - in `vscode` install the plugin `rust-analyzer`
 - in `vscode` activate format-on-save via `File` > `Preferences` > `Settings` then check the box for "Format on Save"
 - install Criterion with `cargo install cargo-criterion`

## Cheatsheet

 - To test, use `cargo test [start_of_test_name]`. Or, for a complete and much slower build, run `make test`.
 - To generate and view API documentation, use `make doc`.
 - To run, use `make run`.
 - To lint, use `make lint`.
 - To format, use `make format`.
 - To check your code for errors, but skip code generation, use `make check`.  This should be faster than `make build`.
 - To build, use `make build`.
 - To install, use `make install`.
 - To run lint, compile, and run tests use `make all`. Note that this does *not* run install.
 - To run the benchmarks and generate the benchmark report, use `make bench`, or run `cargo criterion --bench <specific-benchmark>`.

## Notes

The `Makefile` recipes set the flag `RUSTFLAGS=-Dwarnings` and this makes the recompilation **much** slower than without this flag, as `cargo` for some reason rebuilds the entire crate when this flag is set and a minor change is made in a test. So it is much faster to run the tests using cargo and then use the `make test` command before e.g. committing to ensure that the test build does not produce any warnings.

## Release protocol

While twenty-first's version is `0.x.y`, releasing a new version:

1. Is the release backwards-compatible?
   Then the new version is `0.x.y+1`. Otherwise the new version is `0.x+1.0`.
2. Create a commit that increases `version = "0.x.y"` in twenty-first/Cargo.toml.
   The commit message should give a one-line summary of each release change.
3. Have a `v0.x.y` [git tag][tag] on this commit created. (`git tag v0.x.y [sha]`, `git push upstream --tags`)
4. Have this commit `cargo publish`ed on [crates.io][crates] and in GitHub [tags][tags].

[tag]: https://git-scm.com/book/en/v2/Git-Basics-Tagging
[tags]: https://github.com/Neptune-Crypto/twenty-first/tags
[crates]: https://crates.io/crates/twenty-first/versions

If you do not have the privilege to create git tags or run `cargo publish`, submit a PR and the merger will take care of these.
