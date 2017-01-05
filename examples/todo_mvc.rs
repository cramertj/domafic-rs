extern crate domafic;
use domafic::{DOMNode, KeyIter, IntoNode, empty};
use domafic::tags::*;
use domafic::listener::on;
use domafic::web_render::run;

enum Msg {
    Add(String),
    Remove,
    None,
}

fn main() {

    let update = |state: &mut Vec<String>, msg: Msg, mut keys: KeyIter| {
        match msg {
            Msg::Add(todo) => state.push(todo),
            Msg::Remove => {
                state.remove(keys.next().unwrap());
            },
            Msg::None => {},
        }
    };

    let render_item = |state: &str| {
        div ((
            state.to_owned().into_node(),
            button ((
                on("click", |_| Msg::Remove),
                "Remove".into_node(),
            )),
        ))
    };

    const ENTER_KEYCODE: i32 = 13;
    let render = |state: &Vec<String>| {
        div ((
            h1("TODO:".into_node()),
            input((
                attributes([("placeholder", "What do you have to do?")]),
                on("keydown", |event|
                    if let (ENTER_KEYCODE, Some(target_value)) =
                        (event.which_keycode, event.target_value)
                    {
                        Msg::Add(target_value.to_owned())
                    } else { Msg::None }
                ),
                empty()
            )),
            state
                .iter().enumerate()
                .map(|(index, todo)| render_item(todo).with_key(index))
                .collect::<Vec<_>>()
        ))
    };

    run("body", update, render, Vec::new());
}
