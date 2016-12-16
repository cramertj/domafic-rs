trait DOMNode {}
impl<'a, T: DOMNode> DOMNode for &'a T {}

trait DOMLevel {
    type NodeType: DOMNode;
    type ChildrenType: DOMChildren;
}
impl<T: DOMNode> DOMLevel for T {
    type NodeType = T;
    type ChildrenType = ();
}

trait DOMChildrenProcessor {
    type State;
    fn get_processor<T: DOMLevel>() -> fn(&mut Self::State, &T) -> ();
}

trait DOMChildren {
    fn process_all<P: DOMChildrenProcessor>(&self, processor: &mut P::State) -> ();
}
impl DOMChildren for () {
    fn process_all<P: DOMChildrenProcessor>(&self, _processor: &mut P::State) -> () {}
}

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
            where $typ: DOMLevel + Copy,
                  $( $ntyp: DOMLevel + Copy ),*
        {
            fn process_all<P>(&self, state: &mut P::State) -> ()
                    where P: DOMChildrenProcessor {
                P::get_processor()(state, &self.$idx);
                $(
                    P::get_processor()(state, &self.$nidx);
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

    #[derive(Copy, Clone)]
    struct BogusOne;
    impl DOMNode for BogusOne {}

    struct BogusTwo;
    impl DOMNode for BogusTwo {}

    struct ChildCounter;
    impl DOMChildrenProcessor for ChildCounter {
        type State = usize;

        fn get_processor<T: DOMLevel>() -> fn(&mut Self::State, &T) -> () {
            fn incr<T: DOMLevel>(state: &mut usize, _level: &T) {
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
    }
}
