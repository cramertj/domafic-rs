use {DomNode, DomNodes, DomValue};
use processors::DomNodeProcessor;

use std::marker::PhantomData;
use std::fmt;
use std::io;

/// Type to use for processing a `DomNode` tree and writing it to HTML.
///
/// This type should not ever need to be instantiated. Instead, simply
/// name the type in calls to `DomNodes::process_all::<HtmlWriter<...>>(...)`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct HtmlWriter<W: io::Write>(PhantomData<W>);
impl<'a, M, W: io::Write> DomNodeProcessor<'a, M> for HtmlWriter<W> {
    type Acc = W;
    type Error = io::Error;

    fn get_processor<T: DomNode>() -> fn(&mut Self::Acc, &T) -> Result<(), Self::Error> {
        fn add_node<W, T>(w: &mut W, node: &T) -> Result<(), io::Error>
                where W: io::Write, T: DomNode {
            match node.value() {
                DomValue::Element { tag: tagname } => {
                    write!(w, "<{}", tagname)?;
                    for attr in node.attributes() {
                        write!(w, " {}=\"{}\"", attr.0, attr.1)?;
                    }
                    write!(w, ">")?;
                    node.children().process_all::<HtmlWriter<W>>(w)?;
                    write!(w, "</{}>", tagname)
                }
                DomValue::Text(text) => {
                    write!(w, "{}", text)
                }
            }
        }
        add_node
    }
}

/// Wrapper struct to allow `DomNode`s to implement `Display` as html
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct HtmlDisplayable<'a, T: DomNode + 'a>(pub &'a T);

impl<'a, T: DomNode> fmt::Display for HtmlDisplayable<'a, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // TODO the extra string allocation here is almost certainly avoidable
        let mut string_buffer = Vec::new();
        self.0.write_html(&mut string_buffer)
            .map_err(|_| fmt::Error)?;
        let string = String::from_utf8(string_buffer)
            .map_err(|_| fmt::Error)?;
        formatter.write_str(&string)
    }
}
