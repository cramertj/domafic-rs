// TEMPORARY Test with asmjs
// TODO Replace with examples in /examples once Windows builds are fixed
// so that the manual `cargo rustc ... --linker="emcc.bat"` workaround is
// unnecessary

extern crate domafic;
use domafic::{DOMNode, KeyIter, IntoNode};
use domafic::tags::{button, div, h1};
use domafic::events::EventType::Click;
use domafic::listener::on;
use domafic::web_render::run;

enum Msg {
    Increment,
    Decrement,
}

fn main() {

    let update_button = |state: &mut isize, msg: Msg, _keys: KeyIter| {
        *state = match msg {
            Msg::Increment => *state + 1,
            Msg::Decrement => *state - 1,
        }
    };

    let decrement = &|_| Msg::Decrement;
    let increment = &|_| Msg::Increment;

    let render_button = |state: &isize| {
        div ((
            button ((
                on(Click, decrement),
                "-".into_node(),
            )),
            state.to_string().into_node(),
            button ((
                on(Click, increment),
                "+".into_node(),
            )),
        ))
    };

    let update = |state: &mut Vec<isize>, msg: Msg, mut keys: KeyIter| {
        let key = keys.next().unwrap();
        update_button(&mut state[key], msg, keys)
    };

    let render = |state: &Vec<isize>| {
        div ((
            h1("Hello from rust!".into_node()),
            state
                .iter().enumerate()
                .map(|(index, count)| render_button(count).with_key(index))
                .collect::<Vec<_>>()
        ))
    };

    run("body", update, render, vec![0; 2000]);
}
