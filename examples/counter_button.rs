extern crate domafic;
use domafic::IntoNode;
use domafic::tags::{button, div, h1};
use domafic::listener::on;

// If rendering client-side with asm.js or WebAssembly:
#[cfg(target_os = "emscripten")]
use domafic::web_render::run;
#[cfg(target_os = "emscripten")]
use domafic::KeyIter;

type State = isize;

enum Msg {
    Increment,
    Decrement,
}

fn main() {
    #[cfg(target_os = "emscripten")]
    let update = |state: &mut State, msg: Msg, _keys: KeyIter| {
        *state = match msg {
            Msg::Increment => *state + 1,
            Msg::Decrement => *state - 1,
        }
    };

    let render = |state: &State| {
        div ((
            h1("Hello from rust!".into_node()),
            button ((
                on("click", |_| Msg::Decrement),
                "-".into_node(),
            )),
            state.to_string().into_node(),
            button ((
                on("click", |_| Msg::Increment),
                "+".into_node(),
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
