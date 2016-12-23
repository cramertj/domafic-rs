use {DOMNode, DOMValue};
use processors::DOMNodeProcessor;

use std::marker::PhantomData;
use std::io;

pub struct HtmlWriter<W: io::Write>(PhantomData<W>);
impl<W: io::Write> DOMNodeProcessor for HtmlWriter<W> {
    type Acc = W;
    type Error = io::Error;

    fn get_processor<T: DOMNode>() -> fn(&mut Self::Acc, &T) -> Result<(), Self::Error> {
        fn add_node<W, T>(w: &mut W, node: &T) -> Result<(), io::Error>
                where W: io::Write, T: DOMNode {
            match node.value() {
                DOMValue::Element { tag: tagname } => {
                    write!(w, "<{}", tagname)?;
                    for attr in node.attributes() {
                        write!(w, " {}=\"{}\"", attr.0, attr.1)?;
                    }
                    write!(w, ">")?;
                    node.process_children::<HtmlWriter<W>>(w)?;
                    write!(w, "</{}>", tagname)?;
                }
                DOMValue::Text(text) => {
                    // TODO: HTML escaping
                    write!(w, "{}", text)?;
                }
            }
            Ok(())
        }
        add_node
    }
}
