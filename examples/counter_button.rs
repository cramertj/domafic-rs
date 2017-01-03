extern crate domafic;
use domafic::{KeyIter, IntoNode};
use domafic::tags::{button, div, h1};
use domafic::events::EventType::Click;
use domafic::listener::on;

// If rendering client-side with ASM-JS:
// use domafic::web_render::run;

// If rendering server-side:
// use domafic::DOMNodes;
// use domafic::html_writer::HtmlWriter;

type State = isize;

enum Msg {
    Increment,
    Decrement,
}

fn main() {
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

    // If rendering client-side with ASM-JS:
    // run("body", update, render, 0);

    // If rendering server-side:
    // let mut string_buffer = Vec::new();
    // render(0).process_all::<HtmlWriter<Vec<u8>>>(&mut string_buffer).unwrap();
    // let string = String::from_utf8(string_buffer).unwrap();
    // render(0).process_all<HtmlWriter>();
}
