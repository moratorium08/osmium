# osmium

Operating System written in Rust on RISC V(32im)

## Rust version

```
rustc 1.34.0-nightly (f29b4fbd7 2019-01-31)
```

## Setup

1. install rust/cargo.
2. install riscv-toolchain
3.

```
rustup component add rust-src
rustup install nightly
cargo install cargo-xbuild
```

4. `make setup` (install auxiliary tools(emulator and elf2bin translator). This part needs g++, Python3 and pipenv)


## Build

```
make build
```

## Run on the Emulator

```
make run
```
