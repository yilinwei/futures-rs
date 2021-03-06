use crate::sink::{SinkExt, SinkMapErr};
use core::pin::Pin;
use futures_core::stream::Stream;
use futures_core::task::{Context, Poll};
use futures_sink::{Sink};
use pin_utils::unsafe_pinned;

/// Sink for the [`sink_err_into`](super::SinkExt::sink_err_into) method.
#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct SinkErrInto<Si: Sink<Item>, Item, E> {
    sink: SinkMapErr<Si, fn(Si::SinkError) -> E>,
}

impl<Si, E, Item> SinkErrInto<Si, Item, E>
    where Si: Sink<Item>,
          Si::SinkError: Into<E>,
{
    unsafe_pinned!(sink: SinkMapErr<Si, fn(Si::SinkError) -> E>);

    pub(super) fn new(sink: Si) -> Self {
        SinkErrInto {
            sink: SinkExt::sink_map_err(sink, Into::into),
        }
    }

    /// Get a shared reference to the inner sink.
    pub fn get_ref(&self) -> &Si {
        self.sink.get_ref()
    }

    /// Get a mutable reference to the inner sink.
    pub fn get_mut(&mut self) -> &mut Si {
        self.sink.get_mut()
    }

    /// Consumes this combinator, returning the underlying sink.
    ///
    /// Note that this may discard intermediate state of this combinator, so
    /// care should be taken to avoid losing resources when this is called.
    pub fn into_inner(self) -> Si {
        self.sink.into_inner()
    }
}

impl<Si, Item, E> Sink<Item> for SinkErrInto<Si, Item, E>
    where Si: Sink<Item>,
          Si::SinkError: Into<E>,
{
    type SinkError = E;

    delegate_sink!(sink, Item);
}

impl<S, Item, E> Stream for SinkErrInto<S, Item, E>
    where S: Sink<Item> + Stream,
          S::SinkError: Into<E>
{
    type Item = S::Item;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<S::Item>> {
        self.sink().poll_next(cx)
    }
}
