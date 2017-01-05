use {DOMNode, DOMNodes, DOMValue};
use processors::DOMNodeProcessor;

use std::marker::PhantomData;
use std::fmt;
use std::io;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct HtmlWriter<W: io::Write>(PhantomData<W>);
impl<'a, M, W: io::Write> DOMNodeProcessor<'a, M> for HtmlWriter<W> {
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
                    node.children().process_all::<HtmlWriter<W>>(w)?;
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

/// Wrapper struct to allow `DOMNode`s to implement `Display` as html
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct HtmlDisplayable<'a, T: DOMNode + 'a>(pub &'a T);

impl<'a, T: DOMNode> fmt::Display for HtmlDisplayable<'a, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // TODO the extra string allocation here is almost certainly avoidable
        let mut string_buffer = Vec::new();
        self.0.process_all::<HtmlWriter<Vec<u8>>>(&mut string_buffer)
            .map_err(|_| fmt::Error)?;
        let string = String::from_utf8(string_buffer)
            .map_err(|_| fmt::Error)?;
        formatter.write_str(&string)
    }
}
