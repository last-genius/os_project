# Rust Operating System

An attempt at a simple operating system in Rust and a semester project for the Operating Systems course at CS@UCU.

## Pre-requirements

* Properly set-up [Rust](https://www.rust-lang.org/tools/install). We are using nightly, so check that you have that too by running `rustc --version --verbose` in the cloned project directory. Try `rustup override set nightly` if it's not nightly.
* Install QEMU since we are using it to run our operating system

## Usage

Running the project for the first time will require rebuilding the core library for our custom target, so it can take some time.

```
make
make run
```

