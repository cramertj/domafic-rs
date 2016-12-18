
#![feature(conservative_impl_trait)]

pub trait DOMNode: Sized {
    fn get_attribute(&self, _index: usize) -> Option<&KeyValue> { None }
    fn attributes<'a>(&'a self) -> AttributeIter<'a, Self> {
        AttributeIter { node: self, index: 0 }
    }
    fn with_attributes<A: AsRef<[KeyValue]>>(self, attrs: A) -> WithAttributes<Self, A> {
        WithAttributes { node: self, attributes: attrs }
    }
    fn process_children<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error>;
    fn value<'a>(&'a self) -> DOMValue<'a>;
}

type KeyValue = (&'static str, &'static str);

pub enum DOMValue<'a> {
    Element { tag: &'a str },
    Text(&'a str),
}

pub struct WithAttributes<T: DOMNode, A: AsRef<[KeyValue]>> {
    node: T,
    attributes: A,
}

impl<T, A> DOMNode for WithAttributes<T, A> where T: DOMNode, A: AsRef<[KeyValue]> {
    fn get_attribute(&self, index: usize) -> Option<&KeyValue> {
        let attributes = self.attributes.as_ref();
        attributes
            .get(index)
            .or_else(|| self.node.get_attribute(index - attributes.len()))
    }
    fn process_children<P: DOMNodeProcessor>(&self, _acc: &mut P::Acc) -> Result<(), P::Error> {
        Ok(())
    }
    fn value<'a>(&'a self) -> DOMValue<'a> { self.node.value() }
}

pub struct AttributeIter<'a, T: DOMNode + 'a> {
    node: &'a T,
    index: usize,
}

impl<'a, T: DOMNode> Iterator for AttributeIter<'a, T> {
    type Item = &'a (&'static str, &'static str);
    fn next(&mut self) -> Option<Self::Item> {
        let res = self.node.get_attribute(self.index);
        self.index += 1;
        res
    }
}

impl<'a, T: DOMNode> DOMNode for &'a T {
    fn process_children<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        (*self).process_children::<P>(acc)?;
        Ok(())
    }
    fn value<'b>(&'b self) -> DOMValue<'b> { (*self).value() }
}

impl DOMNode for String {
    fn process_children<P: DOMNodeProcessor>(&self, _acc: &mut P::Acc) -> Result<(), P::Error> {
        Ok(())
    }

    fn value<'a>(&'a self) -> DOMValue<'a> { DOMValue::Text(self) }
}

impl DOMNode for &'static str {
    fn process_children<P: DOMNodeProcessor>(&self, _acc: &mut P::Acc) -> Result<(), P::Error> {
        Ok(())
    }
    fn value<'a>(&'a self) -> DOMValue<'a> { DOMValue::Text(self) }
}

pub mod tags {
    use super::{DOMNode, DOMNodeProcessor, DOMNodes, DOMValue, KeyValue};

    pub trait TagProperties {
        type Children: DOMNodes;
        type Attributes: AsRef<[KeyValue]>;
        fn properties(self) -> (Self::Children, Self::Attributes);
    }

    impl<C: DOMNodes> TagProperties for C {
        type Children = Self;
        type Attributes = [KeyValue; 0];
        fn properties(self) -> (Self::Children, Self::Attributes) {
            (
                self,
                [],
            )
        }
    }

    pub struct Attrs<A: AsRef<[KeyValue]>>(pub A);
    impl<C: DOMNodes, A: AsRef<[KeyValue]>> TagProperties for (Attrs<A>, C) {
        type Children = C;
        type Attributes = A;
        fn properties(self) -> (Self::Children, Self::Attributes) {
            (
                self.1,
                (self.0).0,
            )
        }
    }

    pub struct Tag<C: DOMNodes, A: AsRef<[KeyValue]>> {
        tagname: &'static str,
        contents: C,
        attributes: A,
    }

    impl<C: DOMNodes, A: AsRef<[KeyValue]>> DOMNode for Tag<C, A> {
        fn get_attribute(&self, index: usize) -> Option<&KeyValue> {
            self.attributes.as_ref().get(index)
        }
        fn process_children<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
            self.contents.process_all::<P>(acc)?;
            Ok(())
        }
        fn value<'a>(&'a self) -> DOMValue<'a> {
            DOMValue::Element {
                tag: self.tagname,
            }
        }
    }

    macro_rules! impl_tags {
        ($($tagname:ident),*) => { $(
            pub fn $tagname<T: TagProperties>(properties: T) -> Tag<T::Children, T::Attributes> {
                let (contents, attributes) = properties.properties();
                Tag {
                    tagname: stringify!($tagname),
                    contents: contents,
                    attributes: attributes,
                }
            }
        )* }
    }

    impl_tags!(
        a, abbr, acronym, address, applet, area, article, aside, audio, b, base, basefont, bdi,
        bdo, big, blockquote, body, br, button, canvas, caption, center, cite, code, col, colgroup,
        datalist, dd, del, details, dfn, dialog, dir, div, dl, dt, em, embed, fieldset,
        figcaption, figure, font, footer, form, frame, framset, h1, h2, h3, h4, h5, h6, head,
        header, hr, i, iframe, img, input, ins, kbd, keygen, label, legend, li, link, main, map,
        mark, menu, menuitem, meta, meter, nav, noframes, noscript, object, ol, optgroup, option,
        output, p, param, pre, progress, q, rp, rt, ruby, s, samp, script, section, select, small,
        source, span, strike, strong, style, sub, summary, sup, table, tbody, td, textarea, tfoot,
        th, thead, time, title, tr, track, tt, u, ul, var, video, wbr
    );
}

/// Processor of a `DOMNode`
pub trait DOMNodeProcessor {
    /// Accumulator
    type Acc;
    type Error;

    /// Folding function
    fn get_processor<T: DOMNode>() -> fn(&mut Self::Acc, &T) -> Result<(), Self::Error>;
}

pub trait DOMNodes {
    fn process_all<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error>;
}

impl DOMNodes for () {
    fn process_all<P: DOMNodeProcessor>(&self, _acc: &mut P::Acc) -> Result<(), P::Error> {
        Ok(())
    }
}

impl<T: DOMNode> DOMNodes for T {
    fn process_all<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        P::get_processor()(acc, self)
    }
}

impl<T: DOMNodes> DOMNodes for [T] {
    fn process_all<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        for x in self {
            x.process_all::<P>(acc)?;
        }
        Ok(())
    }
}

impl<T: DOMNodes> DOMNodes for Vec<T> {
    fn process_all<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        for x in self {
            x.process_all::<P>(acc)?;
        }
        Ok(())
    }
}

macro_rules! array_impls {
    ($($len:expr,)*) => { $(
        impl<T: DOMNodes> DOMNodes for [T; $len] {
            fn process_all<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
                for x in self {
                    x.process_all::<P>(acc)?;
                }
                Ok(())
            }
        }
    )* }
}

array_impls!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
    17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
);

// Credit to @shepmaster for structure of recursive tuple macro
macro_rules! tuple_impls {
    () => {};

    // Copywrite @shepmaster
    (($idx:tt => $typ:ident), $( ($nidx:tt => $ntyp:ident), )*) => {
        /*
         * Invoke recursive reversal of list that ends in the macro expansion implementation
         * of the reversed list
        */
        tuple_impls!([($idx, $typ);] $( ($nidx => $ntyp), )*);
        tuple_impls!($( ($nidx => $ntyp), )*); // invoke macro on tail
    };

    /*
     * ([accumulatedList], listToReverse); recursively calls tuple_impls until the list to reverse
     + is empty (see next pattern)
    */
    ([$(($accIdx: tt, $accTyp: ident);)+]  ($idx:tt => $typ:ident), $( ($nidx:tt => $ntyp:ident), )*) => {
      tuple_impls!([($idx, $typ); $(($accIdx, $accTyp); )*] $( ($nidx => $ntyp), ) *);
    };

    // Finally expand into the implementation
    ([($idx:tt, $typ:ident); $( ($nidx:tt, $ntyp:ident); )*]) => {
        impl<$typ, $( $ntyp ),*> DOMNodes for ($typ, $( $ntyp ),*)
            where $typ: DOMNodes,
                  $( $ntyp: DOMNodes),*
        {
            fn process_all<P>(&self, acc: &mut P::Acc) -> Result<(), P::Error>
                    where P: DOMNodeProcessor {
                &self.$idx.process_all::<P>(acc)?;
                $(
                    &self.$nidx.process_all::<P>(acc)?;
                )*
                Ok(())
            }
        }
    }
}

tuple_impls!(
    (9 => J),
    (8 => I),
    (7 => H),
    (6 => G),
    (5 => F),
    (4 => E),
    (3 => D),
    (2 => C),
    (1 => B),
    (0 => A),
);

pub mod html_writer {
    use super::{DOMNode, DOMNodeProcessor, DOMValue};
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::tags::*;
    use super::html_writer::*;

    struct BogusOne;
    impl DOMNode for BogusOne {
        fn process_children<P: DOMNodeProcessor>(&self, _acc: &mut P::Acc) -> Result<(), P::Error> {
            Ok(())
        }
        fn value<'a>(&'a self) -> DOMValue<'a> {
            DOMValue::Element { tag: "bogus_tag_one" }
        }
    }

    struct BogusTwo;
    impl DOMNode for BogusTwo {
        fn process_children<P: DOMNodeProcessor>(&self, _acc: &mut P::Acc) -> Result<(), P::Error> {
            Ok(())
        }
        fn value<'a>(&'a self) -> DOMValue<'a> {
            DOMValue::Element { tag: "bogus_tag_two" }
        }
    }

    struct ChildCounter;
    #[derive(Debug, Clone, Copy)]
    enum Never {}
    impl DOMNodeProcessor for ChildCounter {
        type Acc = usize;
        type Error = Never;

        fn get_processor<T: DOMNode>() -> fn(&mut Self::Acc, &T) -> Result<(), Never> {
            fn incr<T: DOMNode>(count: &mut usize, _node: &T) -> Result<(), Never> {
                *count += 1;
                Ok(())
            }
            incr
        }
    }

    fn html_sample() -> impl DOMNode {
        div ((
            BogusOne,
            BogusOne,
            BogusTwo,
            table ((
                "something",
                th (()),
                tr (()),
                tr (()),
            )),
        ))
    }

    #[test]
    fn counts_children() {
        let mut count = 0;
        (BogusOne, &BogusOne, &BogusTwo).process_all::<ChildCounter>(&mut count).unwrap();
        assert_eq!(3, count);

        count = 0;
        (BogusOne, (BogusOne,), BogusOne).process_all::<ChildCounter>(&mut count).unwrap();
        assert_eq!(3, count);

        count = 0;
        [BogusOne, BogusOne, BogusOne].process_all::<ChildCounter>(&mut count).unwrap();
        assert_eq!(3, count);

        count = 0;
        (BogusOne, BogusOne,
            [BogusOne, BogusOne, BogusOne],
            [(BogusOne)],
            vec![(), (), ()],
            [&BogusTwo, &BogusTwo, &BogusTwo],
        ).process_all::<ChildCounter>(&mut count).unwrap();
        assert_eq!(9, count);

        let sample = html_sample();

        count = 0;
        sample.process_all::<ChildCounter>(&mut count).unwrap();
        assert_eq!(1, count);

        count = 0;
        sample.process_children::<ChildCounter>(&mut count).unwrap();
        assert_eq!(4, count);
    }

    #[test]
    fn builds_string() {
        let mut string_buffer = Vec::new();
        html_sample().process_all::<HtmlWriter<Vec<u8>>>(&mut string_buffer).unwrap();
        let string = String::from_utf8(string_buffer).unwrap();
        assert_eq!(
            r#"
            <div>
                <bogus_tag_one></bogus_tag_one>
                <bogus_tag_one></bogus_tag_one>
                <bogus_tag_two></bogus_tag_two>
                <table>
                    something
                    <th></th>
                    <tr></tr>
                    <tr></tr>
                </table>
            </div>
            "#.chars().filter(|c| !c.is_whitespace()).collect::<String>(),
            string.to_lowercase()
        );
    }

    fn check_attribute_list<T: DOMNode>(div: T) {
        assert_eq!(div.get_attribute(0), Some(&("attr1", "val1")));
        assert_eq!(div.get_attribute(1), Some(&("attr2", "val2")));
        assert_eq!(div.get_attribute(2), Some(&("attr3", "val3")));
        assert_eq!(div.get_attribute(3), None);

        let mut attr_iter = div.attributes();
        assert_eq!(attr_iter.next(), Some(&("attr1", "val1")));
        assert_eq!(attr_iter.next(), Some(&("attr2", "val2")));
        assert_eq!(attr_iter.next(), Some(&("attr3", "val3")));
        assert_eq!(attr_iter.next(), None);
    }

    #[test]
    fn builds_attribute_list() {
        let div1 = div(())
            .with_attributes([("attr2", "val2"), ("attr3", "val3")])
            .with_attributes([("attr1", "val1")]);
        check_attribute_list(div1);

        let div2 = div((
            Attrs([("attr2", "val2"), ("attr3", "val3")]),
            div(())
        )).with_attributes([("attr1", "val1")]);
        check_attribute_list(div2);
    }
}
