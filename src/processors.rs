use {DOMNode, Listener};

// Note: without an extension to HTRBs, I don't know of a way to make the following traits generic
// enough to prevent duplication (need to be able to be generic on the `DOMNode`/`Listener` bounds)

/// `DOMNodeProcessor`s are used to iterate over `DOMNode`s which may or may not be the same type.
/// Implementations of this trait resemble traditional `fold` functions, modifying an accumulator
/// (of type `Acc`) and returning an error as necessary.
pub trait DOMNodeProcessor {

    /// Type of the accumulator updated by `get_processor`
    type Acc;

    /// Type of error returned by failed calls to `get_processor`
    type Error;

    /// Returns a folding function capable of processing elements of type `T: DOMNode`.
    ///
    /// TODO: Example
    fn get_processor<T: DOMNode>() -> fn(&mut Self::Acc, &T) -> Result<(), Self::Error>;
}

pub trait DOMNodes {
    type Message;
    fn process_all<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error>;
}

/// `ListenerProcessor`s are used to iterate over `Listeners`s which may or may not be the same
/// type. Implementations of this trait resemble traditional `fold` functions, modifying an
/// accumulator (of type `Acc`) and returning an error as necessary.
pub trait ListenerProcessor<Message> {

    /// Type of the accumulator updated by `get_processor`
    type Acc;

    /// Type of error returned by failed calls to `get_processor`
    type Error;

    /// Returns a folding function capable of processing elements of type `T: DOMNode`.
    ///
    /// TODO: Example
    fn get_processor<T: Listener<Message=Message>>() -> fn(&mut Self::Acc, &T) -> Result<(), Self::Error>;
}

pub trait Listeners {
    type Message;
    fn process_all<P: ListenerProcessor<Self::Message>>(&self, acc: &mut P::Acc) -> Result<(), P::Error>;
}

impl<T: DOMNode> DOMNodes for T {
    type Message = T::Message;
    fn process_all<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        P::get_processor()(acc, self)
    }
}

impl<T: Listener> Listeners for T {
    type Message = T::Message;
    fn process_all<P: ListenerProcessor<Self::Message>>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        P::get_processor()(acc, self)
    }
}

impl<T: DOMNodes> DOMNodes for Option<T> {
    type Message = T::Message;
    fn process_all<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        if let Some(ref inner) = *self {
            inner.process_all::<P>(acc)
        } else {
            Ok(())
        }
    }
}

impl<L: Listeners> Listeners for Option<L> {
    type Message = L::Message;
    fn process_all<P: ListenerProcessor<Self::Message>>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        if let Some(ref inner) = *self {
            inner.process_all::<P>(acc)
        } else {
            Ok(())
        }
    }
}

impl<T: DOMNodes> DOMNodes for [T] {
    type Message = T::Message;
    fn process_all<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        for x in self {
            x.process_all::<P>(acc)?;
        }
        Ok(())
    }
}

impl<T: Listeners> Listeners for [T] {
    type Message = T::Message;
    fn process_all<P: ListenerProcessor<Self::Message>>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        for x in self {
            x.process_all::<P>(acc)?;
        }
        Ok(())
    }
}

#[cfg(any(feature = "use_std", test))]
impl<T: DOMNodes> DOMNodes for Vec<T> {
    type Message = T::Message;
    fn process_all<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        for x in self {
            x.process_all::<P>(acc)?;
        }
        Ok(())
    }
}

#[cfg(any(feature = "use_std", test))]
impl<T: Listeners> Listeners for Vec<T> {
    type Message = T::Message;
    fn process_all<P: ListenerProcessor<Self::Message>>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        for x in self {
            x.process_all::<P>(acc)?;
        }
        Ok(())
    }
}

macro_rules! array_impls {
    ($($len:expr,)*) => { $(
        impl<T: DOMNodes> DOMNodes for [T; $len] {
            type Message = T::Message;
            fn process_all<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
                for x in self {
                    x.process_all::<P>(acc)?;
                }
                Ok(())
            }
        }

        impl<T: Listeners> Listeners for [T; $len] {
            type Message = T::Message;
            fn process_all<P: ListenerProcessor<Self::Message>>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
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
                  $( $ntyp: DOMNodes<Message=$typ::Message>),*
        {
            type Message = $typ::Message;
            fn process_all<P>(&self, acc: &mut P::Acc) -> Result<(), P::Error>
                    where P: DOMNodeProcessor {
                &self.$idx.process_all::<P>(acc)?;
                $(
                    &self.$nidx.process_all::<P>(acc)?;
                )*
                Ok(())
            }
        }

        impl<$typ, $( $ntyp ),*> Listeners for ($typ, $( $ntyp ),*)
            where $typ: Listeners,
                  $( $ntyp: Listeners<Message=$typ::Message>),*
        {
            type Message = $typ::Message;
            fn process_all<P>(&self, acc: &mut P::Acc) -> Result<(), P::Error>
                    where P: ListenerProcessor<Self::Message> {
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

#[cfg(feature = "use_either_n")]
mod either_impls {
    use super::{DOMNodes, DOMNodeProcessor, Listeners, ListenerProcessor};

    extern crate either_n;
    use self::either_n::*;

    macro_rules! impl_enums {
        () => {};

        (($enum_name_head:ident, $n_head:ident),
        $(($enum_name_tail:ident, $n_tail:ident),)*) => {

            impl<$n_head, $( $n_tail ),*> DOMNodes for
                $enum_name_head<$n_head, $( $n_tail ),*>
                where $n_head: DOMNodes, $( $n_tail: DOMNodes<Message=$n_head::Message> ),*
            {
                type Message = $n_head::Message;
                fn process_all<P>(&self, acc: &mut P::Acc) -> Result<(), P::Error>
                        where P: DOMNodeProcessor {
                    match *self {
                        $enum_name_head::$n_head(ref value) =>
                            value.process_all::<P>(acc)?,
                        $(
                            $enum_name_head::$n_tail(ref value) =>
                                value.process_all::<P>(acc)?
                        ),*
                    };
                    Ok(())
                }
            }

            impl<$n_head, $( $n_tail ),*> Listeners for
                $enum_name_head<$n_head, $( $n_tail ),*>
                where $n_head: Listeners, $( $n_tail: Listeners<Message=$n_head::Message> ),*
            {
                type Message = $n_head::Message;
                fn process_all<P>(&self, acc: &mut P::Acc) -> Result<(), P::Error>
                        where P: ListenerProcessor<Self::Message> {
                    match *self {
                        $enum_name_head::$n_head(ref value) =>
                            value.process_all::<P>(acc)?,
                        $(
                            $enum_name_head::$n_tail(ref value) =>
                                value.process_all::<P>(acc)?
                        ),*
                    };
                    Ok(())
                }
            }

            impl_enums!($( ($enum_name_tail, $n_tail), )*);
        }
    }

    impl_enums!(
        (Either8, Eight),
        (Either7, Seven),
        (Either6, Six),
        (Either5, Five),
        (Either4, Four),
        (Either3, Three),
        (Either2, Two),
        (Either1, One),
    );
}
