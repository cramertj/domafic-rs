extern crate domafic;

#[cfg(not(target_os = "emscripten"))]
fn main() {
    panic!("This example needs to be run in the browser via the asm.js or WebAssembly targets.")
}

#[cfg(target_os = "emscripten")]
fn main() {
    use domafic::tags::{button, div, h1};
    use domafic::listener::on;

    // If rendering client-side with asm.js or WebAssembly:
    #[cfg(target_os = "emscripten")]
    use domafic::web_render::{run, JsIo, HttpRequest, HttpResult};

    #[derive(Debug, Clone)]
    struct State {
        request_out: bool,
        last_response: Option<String>,
    }

    enum Msg {
        Echo(String),
        #[allow(dead_code)]
        Received(String),
    }

    #[cfg(target_os = "emscripten")]
    let update = |state: &mut State, msg: Msg, _keys, js_io: &JsIo<Msg>| {
        match msg {
            Msg::Echo(message) => {
                js_io.http(HttpRequest {
                    method: "POST",
                    headers: &[("key1", "value1"), ("key2", "value2"), ("key3", "value3")],
                    url: "https://httpbin.org/post",
                    body: &message,
                    timeout_millis: None,
                }, Box::new(|response: HttpResult|
                    Msg::Received(format!("{:?}", response))
                ));
                state.request_out = true;
            }
            Msg::Received(received) => {
                state.request_out = false;
                state.last_response = Some(received);
            }
        };
    };

    let render = |state: &State| {
        div ((
            h1("Hello from rust!"),
            button ((
                on("click", |_| Msg::Echo("Message!".into())),
                "Send request",
            )),
            format!("State: {:?}", state),
        ))
    };

    // If rendering server-side:
    #[cfg(not(target_os = "emscripten"))]
    println!("HTML: {}", render(&State {
        request_out: false,
        last_response: None,
    }));

    // If rendering client-side with asm.js or WebAssembly:
    #[cfg(target_os = "emscripten")]
    run("body", update, render, State {
        request_out: false,
        last_response: None,
    });
}
