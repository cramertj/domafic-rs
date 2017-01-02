// TEMPORARY Test with asmjs
// TODO Replace with examples in /examples once Windows builds are fixed
// so that the manual `cargo rustc ... --linker="emcc.bat"` workaround is
// unnecessary
#![allow(unused_unsafe)]

extern crate domafic;
use domafic::IntoNode;
use domafic::tags::{button, div, h1};
use domafic::events::EventType::Click;
use domafic::listener::on;
use domafic::web_render::run;

type State = isize;

enum Msg {
    Increment,
    Decrement,
}

fn main() {
    let update = |state: &mut State, msg: Msg| {
        *state = match msg {
            Msg::Increment => *state + 1,
            Msg::Decrement => *state - 1,
        }
    };

    let render = |state: &State| {
        div ((
            h1("Hello from rust!".into_node()),
            button ((
                on(Click, |_| Msg::Decrement),
                "-".into_node(),
            )),
            state.to_string().into_node(),
            button ((
                on(Click, |_| Msg::Increment),
                "+".into_node(),
            )),
        ))
    };

    run("body", update, render, 0);
}
