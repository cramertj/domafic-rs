extern crate domafic;
use domafic::tags::{button, div, h1};
use domafic::listener::on;

// If rendering client-side with asm.js or WebAssembly:
#[cfg(target_os = "emscripten")]
use domafic::web_render::{run, JsIo};
#[cfg(target_os = "emscripten")]
use domafic::KeyIter;

type State = isize;

enum Msg {
    Increment,
    Decrement,
}

fn main() {
    #[cfg(target_os = "emscripten")]
    let update = |state: &mut State, msg: Msg, _: KeyIter, _: &JsIo<Msg>| {
        *state = match msg {
            Msg::Increment => *state + 1,
            Msg::Decrement => *state - 1,
        }
    };

    let render = |state: &State| {
        div ((
            h1("Hello from rust!"),
            button ((
                on("click", |_| Msg::Decrement),
                "-",
            )),
            state.to_string(),
            button ((
                on("click", |_| Msg::Increment),
                "+",
            )),
        ))
    };

    // If rendering server-side:
    #[cfg(not(target_os = "emscripten"))]
    println!("HTML: {}", render(&0));

    // If rendering client-side with asm.js or WebAssembly:
    #[cfg(target_os = "emscripten")]
    run("body", update, render, 0);
}
