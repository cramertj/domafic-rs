#![cfg_attr(test, feature(conservative_impl_trait))]
#![cfg_attr(not(any(feature = "use_std", test)), no_std)]
#![allow(unused_unsafe)]

/// A `KeyValue` is a pair of static strings corresponding to a mapping between a key and a value.
pub type KeyValue = (&'static str, &'static str);

pub mod dom_node;
pub use dom_node::{DOMNode, DOMValue, IntoNode};
pub mod events;

#[cfg(any(feature = "use_std", test))]
pub mod html_writer;

pub mod key_stack;
pub mod listener;
pub use listener::{Listener, on};
pub mod processors;
pub use processors::{DOMNodes, Listeners};
pub mod tags;

pub use empty::{empty};
pub mod empty {
    #[cfg(not(any(feature = "use_std", test)))]
    extern crate core as std;
    use std::marker::PhantomData;

    use super::processors::{DOMNodes, DOMNodeProcessor, Listeners, ListenerProcessor};

    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
    pub struct EmptyNodes<Message>(PhantomData<Message>);
    pub fn empty<Message>() -> EmptyNodes<Message> { EmptyNodes(PhantomData) }
    impl<M> DOMNodes for EmptyNodes<M> {
        type Message = M;
        fn process_all<'a, P: DOMNodeProcessor<'a, M>>(&'a self, _acc: &mut P::Acc) -> Result<(), P::Error> {
            Ok(())
        }
    }

    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
    pub struct EmptyListeners<Message>(PhantomData<Message>);
    pub fn empty_listeners<Message>() -> EmptyListeners<Message> { EmptyListeners(PhantomData) }
    impl<M> Listeners for EmptyListeners<M> {
        type Message = M;
        fn process_all<'a, P: ListenerProcessor<'a, Self::Message>>(&self, _acc: &mut P::Acc) -> Result<(), P::Error> {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DOMNode, DOMValue, KeyValue, IntoNode};
    use super::tags::*;
    use super::processors::{DOMNodes, DOMNodeProcessor, ListenerProcessor};
    use super::empty::empty;
    use super::html_writer::HtmlWriter;

    #[cfg(feature = "use_either_n")]
    extern crate either_n;
    #[cfg(feature = "use_either_n")]
    use self::either_n::*;

    struct BogusOne;
    impl DOMNode for BogusOne {
        type Message = Never;
        fn key(&self) -> Option<usize> { None }
        fn get_attribute(&self, _index: usize) -> Option<&KeyValue> { None }
        fn process_listeners<P: ListenerProcessor<Self::Message>>(&self, _acc: &mut P::Acc) -> Result<(), P::Error> {
            Ok(())
        }
        fn process_children<P: DOMNodeProcessor>(&self, _acc: &mut P::Acc) -> Result<(), P::Error> {
            Ok(())
        }
        fn value<'a>(&'a self) -> DOMValue<'a> {
            DOMValue::Element { tag: "bogus_tag_one" }
        }
    }

    struct BogusTwo;
    impl DOMNode for BogusTwo {
        type Message = Never;
        fn key(&self) -> Option<usize> { None }
        fn get_attribute(&self, _index: usize) -> Option<&KeyValue> { None }
        fn process_listeners<P: ListenerProcessor<Self::Message>>(&self, _acc: &mut P::Acc) -> Result<(), P::Error> {
            Ok(())
        }
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
    impl<M> DOMNodeProcessor<M> for ChildCounter {
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

    fn html_sample() -> impl DOMNode<Message = Never> {
        div ((
            attributes([("attr", "value")]),
            (
            BogusOne,
            BogusOne,
            BogusTwo,
            table ((
                "something".into_node(),
                th (empty()),
                tr (empty()),
                tr (empty()),
            )),
            )
        ))
    }

    #[cfg(feature = "use_either_n")]
    fn html_either(include_rows: bool) -> impl DOMNode<Message = Never> {
        div((
            table((
                if include_rows {
                    Either2::One((
                        tr("a".into_node()),
                        tr("b".into_node()),
                    ))
                } else {
                    Either2::Two("sumthin else".into_node())
                }
            ))
        ))
    }

    #[cfg(feature = "use_either_n")]
    fn builds_an_either_string(arg: bool, expected: &'static str) {
        let mut string_buffer = Vec::new();
        html_either(arg).process_all::<HtmlWriter<Vec<u8>>>(&mut string_buffer).unwrap();
        let string = String::from_utf8(string_buffer).unwrap();
        assert_eq!(
            without_whitespace(expected.to_string()),
            without_whitespace(string)
        );
    }

    #[cfg(feature = "use_either_n")]
    #[test]
    fn builds_either_string() {
        builds_an_either_string(true, r#"
        <div>
            <table>
                <tr>a</tr>
                <tr>b</tr>
            </table>
        </div>
        "#);

        builds_an_either_string(false, r#"
        <div>
            <table>
                sumthin else
            </table>
        </div>
        "#);
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
            vec![empty(), empty(), empty()],
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

    fn without_whitespace(string: String) -> String {
        string.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn builds_string() {
        let mut string_buffer = Vec::new();
        html_sample().process_all::<HtmlWriter<Vec<u8>>>(&mut string_buffer).unwrap();
        let string = String::from_utf8(string_buffer).unwrap();
        assert_eq!(
            without_whitespace(r#"
            <div attr="value">
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
            "#.to_string()),
            without_whitespace(string)
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
        let div1 = div(empty::<Never>())
            .with_attributes([("attr2", "val2"), ("attr3", "val3")])
            .with_attributes([("attr1", "val1")]);
        check_attribute_list(div1);

        let div2 = div((
            attributes([("attr2", "val2"), ("attr3", "val3")]),
            div(empty::<Never>())
        )).with_attributes([("attr1", "val1")]);
        check_attribute_list(div2);
    }
}
