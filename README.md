# Bringing Rust to Safety-Critical Systems in Space

The development of safety-critical aerospace systems
is traditionally dominated by the C language. Its language characteristics make it trivial to accidentally introduce memory safety
issues resulting in undefined behavior or security vulnerabilities.
The Rust language aims to drastically reduce the chance of
introducing bugs and consequently produces overall more secure
and safer code.

## Contributions
1. We evaluate the state of the Rust ecosystem for use in
safety-critical systems in space.
2. We present a process to partially replace components
of software developed in C with equivalent Rust implementations.
3. We identify and patch three security issues in the
Cubesat Space Protocol, a popular open-source packet
communication protocol for satellites.
4. We develop a new target configuration for the Rust
compiler, allowing the compilation of programs for bare
metal PowerPC CPUs.
5. Based on the combined insights of our contributions,
we develop a set of recommendations for practitioners
in the realms of space system development.

This repository is still WIP and we will add source code and notes on the remaining contributions soon.

## libcsp-rs
### Partial Rewrite

We perform a partial rewrite of critical sections in [libCSP's C implementation](https://github.com/libcsp/libcsp) to demonstrate how Rust can be gradually incorporated into existing C code bases.

#### How to Build
Build the static Rust lib and make sure to build `core::` with `panic = "abort"`.
Building the lib will otherwise still succeed but linking it into a program later on will fail due to
missing refs to `rust_eh_personality`.
```bash
cargo build --release
```

Note that the compiled lib will be in `./target/powerpc-unknown-none/release/libcsp_rs.a`.

The flag to build the core lib and the target are defined in `.cargo/config.toml`, thus the library automatically builds for PPC.

Other things to note (but that are taken care of):
- `csp-rs` is intended to be used as part of `libcsp`, i.e., you should link against `libcsp` as a dependency. 
    Linking directly against `csp-rs` is not tested.
- FFI bindings were generated with `rust-bindgen` from the `wrapper.h` header:
    ```
    bindgen wrapper.h -o libcsp_ffi.rs --use-core --no-layout-tests
    ```
    (note the additional flags to make sure that we end up being `#![no_std]` compliant)
- C bindings to our Rust lib are included in `./libcsp/src/libcsp_rs.h`
- `csp_can1_rx` was replaced with an include and an `extern` ref in `libcsp/src/interfaces/csp_if_can.c`

### Security Analysis
We identified and fixed multiple bugs in libCSP in collaboration with the maintainers:
https://github.com/libcsp/libcsp/pull/510

## Citation
```json
@inproceedings{rustspace3S,
    author = {Seidel, Lukas and Beier, Julian},
    title = {Bringing Rust to Safety-Critical Systems in Space},
    booktitle = {IEEE Security for Space Systems (3S)},
    year = {2024},
    month = {May},
}
```
