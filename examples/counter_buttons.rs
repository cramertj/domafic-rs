extern crate domafic;
use domafic::{DOMNode, KeyIter, IntoNode};
use domafic::tags::{button, div, h1};
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

    let render_button = |state: &isize| {
        div ((
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

    run("body", update, render, vec![0; 10]);
}
