# Domafic - Safe, high-performance, universal web applications

[![Build Status](https://travis-ci.org/cramertj/domafic-rs.svg?branch=master)](https://travis-ci.org/cramertj/domafic-rs)
[![crates.io](https://img.shields.io/crates/v/domafic.svg)](https://crates.io/crates/domafic)

[Documentation](https://docs.rs/domafic)

## Installing Emscripten
Using Domafic in the browser requires Emscripten.
To get started with Emscripten, follow the steps detailed
[here.](https://users.rust-lang.org/t/compiling-to-the-web-with-rust-and-emscripten)

## Running the Examples
To try the examples in a browser, start by compiling the example to asm.js:
`cargo build --example todo_mvc --target=asmjs-unknown-emscripten`
If this is your first time compiling with Emscripten, this may take a while.
Once the example is built, open up `index_debug.html` (for debug builds) or
`index_release.html` (for release builds) and make sure the script `src` is
set to point at the example you want to run. From there it's as simple as
opening up your browser and trying it out!
