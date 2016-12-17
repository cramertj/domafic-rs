pub trait DOMNode {
    type ChildrenType: DOMChildren;
    fn children(&self) -> &Self::ChildrenType;
    fn value<'a>(&'a self) -> DOMValue<'a>;
}
pub enum DOMValue<'a> {
    Element { tag: &'a str },
    Text(&'a str),
}

impl<'a, T: DOMNode> DOMNode for &'a T {
    type ChildrenType = T::ChildrenType;
    fn children(&self) -> &Self::ChildrenType { (*self).children() }
    fn value<'b>(&'b self) -> DOMValue<'b> { (*self).value() }
}

const NONE_REF: &'static () = &();

impl DOMNode for String {
    type ChildrenType = ();
    fn children(&self) -> &Self::ChildrenType { NONE_REF }
    fn value<'a>(&'a self) -> DOMValue<'a> { DOMValue::Text(self) }
}

impl<'a> DOMNode for &'a str {
    type ChildrenType = ();
    fn children(&self) -> &Self::ChildrenType { NONE_REF }
    fn value<'b>(&'b self) -> DOMValue<'b> { DOMValue::Text(self) }
}

pub mod tags {
    use super::{DOMNode, DOMChildren, DOMValue};

    macro_rules! impl_tags {
        ($($tagname:ident,)*) => { $(
            pub struct $tagname<C: DOMChildren>(pub C);
            impl<C: DOMChildren> DOMNode for $tagname<C> {
                type ChildrenType = C;
                fn children(&self) -> &Self::ChildrenType { &self.0 }
                fn value<'a>(&'a self) -> DOMValue<'a> {
                    DOMValue::Element {
                        tag: stringify!($tagname)
                    }
                }
            }
        )* }
    }

    impl_tags!(
        A, B, Big, BlockQuote, Body, Br, Center, Del, Div, Em,
        Font, Head, H1, H2, H3, H4, H5, H6, HR, I, Img, Ins,
        Li, Ol, P, Pre, S, Small, Span, Strong, Sub, Sup,
        Table, TD, TH, Title, TR, TT, U, UL,
    );
}

/// Processor of a `DOMNode`
pub trait DOMNodeProcessor {
    /// Accumulator
    type Acc;

    /// Folding function
    fn get_processor<T: DOMNode>() -> fn(&mut Self::Acc, &T) -> ();
}

pub trait DOMChildren {
    fn process_all<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> ();
}

impl DOMChildren for () {
    fn process_all<P: DOMNodeProcessor>(&self, _acc: &mut P::Acc) -> () {}
}

impl<T: DOMNode> DOMChildren for T {
    fn process_all<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> () {
        P::get_processor()(acc, self);
    }
}

impl<T: DOMChildren> DOMChildren for [T] {
    fn process_all<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> () {
        for x in self {
            x.process_all::<P>(acc);
        }
    }
}

impl<T: DOMChildren> DOMChildren for Vec<T> {
    fn process_all<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> () {
        for x in self {
            x.process_all::<P>(acc);
        }
    }
}

macro_rules! array_impls {
    ($($len:expr,)*) => { $(
        impl<T: DOMChildren> DOMChildren for [T; $len] {
            fn process_all<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> () {
                for x in self {
                    x.process_all::<P>(acc);
                }
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
        impl<$typ, $( $ntyp ),*> DOMChildren for ($typ, $( $ntyp ),*)
            where $typ: DOMChildren,
                  $( $ntyp: DOMChildren),*
        {
            fn process_all<P>(&self, acc: &mut P::Acc) -> ()
                    where P: DOMNodeProcessor {
                &self.$idx.process_all::<P>(acc);
                $(
                    &self.$nidx.process_all::<P>(acc);
                )*
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

pub mod html_string {
    use super::{DOMNode, DOMChildren, DOMNodeProcessor, DOMValue};

    pub struct HtmlStringBuilder;
    impl DOMNodeProcessor for HtmlStringBuilder {
        type Acc = String;

        fn get_processor<T: DOMNode>() -> fn(&mut Self::Acc, &T) -> () {
            fn add_node<T: DOMNode>(string: &mut String, node: &T) {
                match node.value() {
                    DOMValue::Element { tag: tagname } => {
                        string.push_str("<");
                        string.push_str(tagname);
                        string.push_str(">");

                        node.children().process_all::<HtmlStringBuilder>(string);

                        string.push_str("</");
                        string.push_str(tagname);
                        string.push_str(">");
                    }
                    DOMValue::Text(text) => {
                        // TODO: HTML escaping
                        string.push_str(text);
                    }
                }
            }
            add_node
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::tags::*;
    use super::html_string::*;

    struct BogusOne;
    impl DOMNode for BogusOne {
        type ChildrenType = ();
        fn children(&self) -> &Self::ChildrenType { NONE_REF }
        fn value<'a>(&'a self) -> DOMValue<'a> {
            DOMValue::Element { tag: "bogus_tag_one" }
        }
    }

    struct BogusTwo;
    impl DOMNode for BogusTwo {
        type ChildrenType = ();
        fn children(&self) -> &Self::ChildrenType { NONE_REF }
        fn value<'a>(&'a self) -> DOMValue<'a> {
            DOMValue::Element { tag: "bogus_tag_two" }
        }
    }

    struct ChildCounter;
    impl DOMNodeProcessor for ChildCounter {
        type Acc = usize;

        fn get_processor<T: DOMNode>() -> fn(&mut Self::Acc, &T) -> () {
            fn incr<T: DOMNode>(count: &mut usize, _node: &T) {
                *count += 1;
            }
            incr
        }
    }

    static HTML_SAMPLE:
        Div<(BogusOne, BogusOne, BogusTwo, Table<(&'static str, TH<()>, TR<()>, TR<()>)>)> =

    Div ((
        BogusOne,
        BogusOne,
        BogusTwo,
        Table ((
            "something",
            TH (()),
            TR (()),
            TR (()),
        )),
    ));

    #[test]
    fn counts_children() {
        let mut count = 0;
        (BogusOne, &BogusOne, &BogusTwo).process_all::<ChildCounter>(&mut count);
        assert_eq!(3, count);

        count = 0;
        (BogusOne, (BogusOne,), BogusOne).process_all::<ChildCounter>(&mut count);
        assert_eq!(3, count);

        count = 0;
        [BogusOne, BogusOne, BogusOne].process_all::<ChildCounter>(&mut count);
        assert_eq!(3, count);

        count = 0;
        (BogusOne, BogusOne,
            [BogusOne, BogusOne, BogusOne],
            [(BogusOne)],
            vec![(), (), ()],
            [&BogusTwo, &BogusTwo, &BogusTwo],
        ).process_all::<ChildCounter>(&mut count);
        assert_eq!(9, count);

        count = 0;
        HTML_SAMPLE.process_all::<ChildCounter>(&mut count);
        assert_eq!(1, count);

        let div_children = HTML_SAMPLE.children();

        count = 0;
        div_children.process_all::<ChildCounter>(&mut count);
        assert_eq!(4, count);
    }

    #[test]
    fn builds_string() {
        let mut string = String::new();
        HTML_SAMPLE.process_all::<HtmlStringBuilder>(&mut string);
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
}
