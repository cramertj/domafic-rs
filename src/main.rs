// TEMPORARY Test with asmjs
// TODO Replace with examples in /examples once Windows builds are fixed
// so that the manual `cargo rustc ... --linker="emcc.bat"` workaround is
// unnecessary

extern crate domafic;
use domafic::{DOMNode, DOMNodes, DOMValue, IntoNode};
use domafic::tags::{button, div, h1};
use domafic::events::EventType::Click;
use domafic::listener::on;
use domafic::processors::DOMNodeProcessor;

extern crate webplatform;

type State = usize;

enum Msg {
    Increment,
    Decrement,
}

fn main() {
    let update = |state: State, msg: Msg| match msg {
        Msg::Increment => state + 1,
        Msg::Decrement => state - 1,
    };

    let render = |state: State| {
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

    let document = webplatform::init();
    let body = document.element_query("body").unwrap();
    render(0)
        .process_all::<WebPlatformWriter>(&mut (&document, &body))
        .unwrap();

    webplatform::spin();
}

use std::marker::PhantomData;
use webplatform::{Document as WebDoc, HtmlNode as WebNode};
pub struct WebPlatformWriter<'a, 'd: 'a, 'n: 'a>(PhantomData<(&'a (), &'d (), &'n ())>);
impl<'a, 'd: 'a, 'n: 'a> DOMNodeProcessor for WebPlatformWriter<'a, 'd, 'n> {
    type Acc = (&'a WebDoc<'d>, &'a WebNode<'n>);
    type Error = ();

    fn get_processor<T: DOMNode>() -> fn(&mut Self::Acc, &T) -> Result<(), Self::Error> {
        fn add_node<'a, 'd, 'n, T>(
                acc: &mut (&'a WebDoc<'d>, &'a WebNode<'n>),
                node: &T)
            -> Result<(), ()> where T: DOMNode
        {
            let (ref document, ref parent_node) = *acc;

            match node.value() {
                DOMValue::Element { tag: tagname } => {
                    let html_node = document.element_create(tagname).unwrap();
                    for attr in node.attributes() {
                        html_node.prop_set_str(attr.0, attr.1);
                    }

                    // Reborrow of *document needed to match lifetimes for 'a
                    let new_acc = &mut (&**document, &html_node);
                    node.process_children::<WebPlatformWriter>(new_acc)?;
                    let (_, html_node) = *new_acc;
                    parent_node.append(&html_node);
                }
                DOMValue::Text(text) => {
                    // TODO replace with createTextNode (need to add to webplatform API)
                    parent_node.html_append(text);
                }
            }
            Ok(())
        }
        add_node
    }
}
