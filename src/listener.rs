use events;
use processors::ListenerProcessor;

#[cfg(not(any(feature = "use_std", test)))]
extern crate core as std;
use std::marker::PhantomData;

pub trait Listener {
    type Message;
    fn event_types_handled<'a>() -> &'a [events::EventType];
    fn handle_event(&self, event: events::Event) -> Self::Message;
}

pub struct MappedListener<'a, M, L: Listener + 'a, F: Map<L::Message, Out=M>>(&'a L, PhantomData<(M, F)>);
impl<'a, M, L: Listener, F: Map<L::Message, Out=M>> Listener for MappedListener<'a, M, L, F> {
    type Message = M;
    fn event_types_handled<'b>() -> &'b [events::EventType] {
        L::event_types_handled()
    }
    fn handle_event(&self, event: events::Event) -> Self::Message {
        F::map(self.0.handle_event(event))
    }
}

pub trait Map<In> {
    type Out;
    fn map(input: In) -> Self::Out;
}

pub struct MappedListenerProcessor<OldM, NewM, L: ListenerProcessor<OldM>, F: Map<NewM, Out=OldM>>
    (L, F, PhantomData<(OldM, NewM)>);

impl<OldM, NewM, L: ListenerProcessor<OldM>, F: Map<NewM, Out=OldM>> ListenerProcessor<NewM>
    for MappedListenerProcessor<OldM, NewM, L, F>
{
    type Acc = L::Acc;
    type Error = L::Error;
    fn get_processor<T: Listener<Message=NewM>>() -> fn(&mut Self::Acc, &T) -> Result<(), Self::Error> {
        fn process_mapped<OldM, NewM, L: ListenerProcessor<OldM>, F: Map<NewM, Out=OldM>,
            T: Listener<Message=NewM>>(acc: &mut L::Acc, listener: &T)
            -> Result<(), L::Error>
        {
            L::get_processor()(acc, &MappedListener::<OldM, T, F>(listener, PhantomData))
        }
        process_mapped::<OldM, NewM, L, F, T>
    }
}
