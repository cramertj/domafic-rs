pub trait DOMNode {
    type ChildrenType: DOMChildren;
    fn children(&self) -> &Self::ChildrenType;
}
impl<'a, T: DOMNode> DOMNode for &'a T {
    type ChildrenType = T::ChildrenType;
    fn children(&self) -> &Self::ChildrenType { (*self).children() }
}

pub mod tags {
    use super::{DOMNode, DOMChildren};

    macro_rules! impl_tags {
        ($($tagname:ident,)*) => { $(
            pub struct $tagname<C: DOMChildren>(pub C);
            impl<C: DOMChildren> DOMNode for $tagname<C> {
                type ChildrenType = C;
                fn children(&self) -> &Self::ChildrenType { &self.0 }
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

/// Processor that can fold over all the children of a `DOMNode`
pub trait DOMChildrenProcessor {
    /// Accumulator
    type Acc;

    /// Folding function
    fn get_processor<T: DOMChildren>() -> fn(&mut Self::Acc, &T) -> ();
}

pub trait DOMChildren {
    fn process_all<P: DOMChildrenProcessor>(&self, acc: &mut P::Acc) -> ();
}

impl DOMChildren for () {
    fn process_all<P: DOMChildrenProcessor>(&self, _acc: &mut P::Acc) -> () {}
}

impl<T: DOMNode> DOMChildren for T {
    fn process_all<P: DOMChildrenProcessor>(&self, acc: &mut P::Acc) -> () {
        P::get_processor()(acc, self);
    }
}

impl<T: DOMChildren> DOMChildren for [T] {
    fn process_all<P: DOMChildrenProcessor>(&self, acc: &mut P::Acc) -> () {
        for x in self {
            x.process_all::<P>(acc);
        }
    }
}

macro_rules! array_impls {
    ($($len:expr,)*) => { $(
        impl<T: DOMChildren> DOMChildren for [T; $len] {
            fn process_all<P: DOMChildrenProcessor>(&self, acc: &mut P::Acc) -> () {
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
                    where P: DOMChildrenProcessor {
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::tags::*;

    static NONE: () = ();

    struct BogusOne;
    impl DOMNode for BogusOne {
        type ChildrenType = ();
        fn children(&self) -> &Self::ChildrenType { &NONE }
    }

    struct BogusTwo;
    impl DOMNode for BogusTwo {
        type ChildrenType = ();
        fn children(&self) -> &Self::ChildrenType { &NONE }
    }

    struct ChildCounter;
    impl DOMChildrenProcessor for ChildCounter {
        type Acc = usize;

        fn get_processor<T: DOMChildren>() -> fn(&mut Self::Acc, &T) -> () {
            fn incr<T: DOMChildren>(state: &mut usize, _level: &T) {
                *state += 1;
            }
            incr
        }
    }

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
            [(), (), ()],
            [&BogusTwo, &BogusTwo, &BogusTwo],
        ).process_all::<ChildCounter>(&mut count);
        assert_eq!(9, count);

        let div = Div ((
            BogusOne,
            BogusOne,
            BogusTwo,
            Table ((
                TH (()),
                TR (()),
                TR (()),
            )),
        ));

        count = 0;
        div.process_all::<ChildCounter>(&mut count);
        assert_eq!(1, count);

        let div_children = div.children();

        count = 0;
        div_children.process_all::<ChildCounter>(&mut count);
        assert_eq!(4, count);
    }
}
