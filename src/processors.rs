use {DomNode, Listener};

use opt_std::marker::PhantomData;

/// `DomNodeProcessor`s are used to iterate over `DomNode`s which may or may not be the same type.
/// Implementations of this trait resemble traditional `fold` functions, modifying an accumulator
/// (of type `Acc`) and returning an error as necessary.
pub trait DomNodeProcessor<'a, Message> {

    /// Type of the accumulator updated by `get_processor`
    type Acc;

    /// Type of error returned by failed calls to `get_processor`
    type Error;

    /// Returns a folding function capable of processing elements of type `T: DomNode`.
    fn get_processor<T: DomNode<Message>>()
        -> fn(&mut Self::Acc, &'a T) -> Result<(), Self::Error>;
}

/// Collection of `DomNode`s with a common message type
pub trait DomNodes<Message> {
    /// Processes all of the `DomNode`s in the given collection using processor `P` and
    /// accumulator `acc`.
    fn process_all<'a, P: DomNodeProcessor<'a, Message>>(&'a self, acc: &mut P::Acc) -> Result<(), P::Error>;
}

/// `ListenerProcessor`s are used to iterate over `Listeners`s which may or may not be the same
/// type. Implementations of this trait resemble traditional `fold` functions, modifying an
/// accumulator (of type `Acc`) and returning an error as necessary.
pub trait ListenerProcessor<'a, Message> {

    /// Type of the accumulator updated by `get_processor`
    type Acc;

    /// Type of error returned by failed calls to `get_processor`
    type Error;

    /// Returns a folding function capable of processing elements of type `T: DomNode`.
    ///
    /// TODO: Example
    fn get_processor<T: Listener<Message>>() -> fn(&mut Self::Acc, &'a T) -> Result<(), Self::Error>;
}

/// Collection of `Listener`s with a common message type
pub trait Listeners<Message> {
    /// Processes all of the listeners in the given collection using processor `P` and
    /// accumulator `acc`.
    fn process_all<'a, P: ListenerProcessor<'a, Message>>(&'a self, acc: &mut P::Acc) -> Result<(), P::Error>;
}

impl<Message> DomNodes<Message> for () {
    fn process_all<'a, P: DomNodeProcessor<'a, Message>>(&'a self, _acc: &mut P::Acc) -> Result<(), P::Error> {
        Ok(())
    }
}

impl<M> DomNodes<M> for PhantomData<M> {
    fn process_all<'a, P: DomNodeProcessor<'a, M>>(&'a self, _acc: &mut P::Acc) -> Result<(), P::Error> {
        Ok(())
    }
}

/// Zero-sized empty collection of listeners
pub struct EmptyListeners;
impl<Message> Listeners<Message> for EmptyListeners {
    fn process_all<'a, P: ListenerProcessor<'a, Message>>(&'a self, _acc: &mut P::Acc) -> Result<(), P::Error> {
        Ok(())
    }
}

impl<Message, T: DomNodes<Message>> DomNodes<Message> for Option<T> {
    fn process_all<'a, P: DomNodeProcessor<'a, Message>>(&'a self, acc: &mut P::Acc) -> Result<(), P::Error> {
        if let Some(ref inner) = *self {
            inner.process_all::<P>(acc)
        } else {
            Ok(())
        }
    }
}

impl<Message, L: Listeners<Message>> Listeners<Message> for Option<L> {
    fn process_all<'a, P: ListenerProcessor<'a, Message>>(&'a self, acc: &mut P::Acc) -> Result<(), P::Error> {
        if let Some(ref inner) = *self {
            inner.process_all::<P>(acc)
        } else {
            Ok(())
        }
    }
}

impl<Message, T: DomNodes<Message>> DomNodes<Message> for [T] {
    fn process_all<'a, P: DomNodeProcessor<'a, Message>>(&'a self, acc: &mut P::Acc) -> Result<(), P::Error> {
        for x in self {
            x.process_all::<P>(acc)?;
        }
        Ok(())
    }
}

impl<Message, T: Listeners<Message>> Listeners<Message> for [T] {
    fn process_all<'a, P: ListenerProcessor<'a, Message>>(&'a self, acc: &mut P::Acc) -> Result<(), P::Error> {
        for x in self {
            x.process_all::<P>(acc)?;
        }
        Ok(())
    }
}

#[cfg(any(feature = "use_std", test))]
impl<Message, T: DomNodes<Message>> DomNodes<Message> for Vec<T> {
    fn process_all<'a, P: DomNodeProcessor<'a, Message>>(&'a self, acc: &mut P::Acc) -> Result<(), P::Error> {
        for x in self {
            x.process_all::<P>(acc)?;
        }
        Ok(())
    }
}

#[cfg(any(feature = "use_std", test))]
impl<Message, T: Listeners<Message>> Listeners<Message> for Vec<T> {
    fn process_all<'a, P: ListenerProcessor<'a, Message>>(&'a self, acc: &mut P::Acc) -> Result<(), P::Error> {
        for x in self {
            x.process_all::<P>(acc)?;
        }
        Ok(())
    }
}

macro_rules! array_impls {
    ($($len:expr,)*) => { $(
        impl<M, T: DomNodes<M>> DomNodes<M> for [T; $len] {
            fn process_all<'a, P: DomNodeProcessor<'a, M>>(&'a self, acc: &mut P::Acc) -> Result<(), P::Error> {
                for x in self {
                    x.process_all::<P>(acc)?;
                }
                Ok(())
            }
        }

        impl<M, T: Listeners<M>> Listeners<M> for [T; $len] {
            fn process_all<'a, P: ListenerProcessor<'a, M>>(&'a self, acc: &mut P::Acc) -> Result<(), P::Error> {
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
        impl<M, $typ, $( $ntyp ),*> DomNodes<M> for ($typ, $( $ntyp ),*)
            where $typ: DomNodes<M>,
                  $( $ntyp: DomNodes<M>),*
        {
            fn process_all<'a, P>(&'a self, acc: &mut P::Acc) -> Result<(), P::Error>
                    where P: DomNodeProcessor<'a, M> {
                &self.$idx.process_all::<P>(acc)?;
                $(
                    &self.$nidx.process_all::<P>(acc)?;
                )*
                Ok(())
            }
        }

        impl<M, $typ, $( $ntyp ),*> Listeners<M> for ($typ, $( $ntyp ),*)
            where $typ: Listeners<M>,
                  $( $ntyp: Listeners<M>),*
        {
            fn process_all<'a, P>(&'a self, acc: &mut P::Acc) -> Result<(), P::Error>
                    where P: ListenerProcessor<'a, M> {
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
    use super::{DomNodes, DomNodeProcessor, Listeners, ListenerProcessor};

    extern crate either_n;
    use self::either_n::*;

    macro_rules! impl_enums {
        () => {};

        (($enum_name_head:ident, $n_head:ident),
        $(($enum_name_tail:ident, $n_tail:ident),)*) => {

            impl<M, $n_head, $( $n_tail ),*> DomNodes<M> for
                $enum_name_head<$n_head, $( $n_tail ),*>
                where $n_head: DomNodes<M>, $( $n_tail: DomNodes<M> ),*
            {
                fn process_all<'a, P>(&'a self, acc: &mut P::Acc) -> Result<(), P::Error>
                        where P: DomNodeProcessor<'a, M> {
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

            impl<M, $n_head, $( $n_tail ),*> Listeners<M> for
                $enum_name_head<$n_head, $( $n_tail ),*>
                where $n_head: Listeners<M>, $( $n_tail: Listeners<M> ),*
            {
                fn process_all<'a, P>(&'a self, acc: &mut P::Acc) -> Result<(), P::Error>
                        where P: ListenerProcessor<'a, M> {
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
