trait DOMNode {
    type ChildrenType: DOMChildren;
}
impl<'a, T: DOMNode> DOMNode for &'a T {
    type ChildrenType = T::ChildrenType;
}

struct Div<C: DOMChildren>(C);
impl<C: DOMChildren> DOMNode for Div<C> {
    type ChildrenType = C;
}

/// Processor that can fold over all the children of a `DOMNode`
trait DOMChildrenProcessor {
    /// Accumulator
    type Acc;

    /// Folding function
    fn get_processor<T: DOMChildren>() -> fn(&mut Self::Acc, &T) -> ();
}

trait DOMChildren {
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
            where $typ: DOMChildren + Copy,
                  $( $ntyp: DOMChildren + Copy ),*
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

    #[derive(Copy, Clone)]
    struct BogusOne;
    impl DOMNode for BogusOne {
        type ChildrenType = ();
    }

    struct BogusTwo;
    impl DOMNode for BogusTwo {
        type ChildrenType = ();
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
    }
}
