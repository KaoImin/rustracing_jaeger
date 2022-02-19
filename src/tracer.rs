#[cfg(not(feature = "sync"))]
pub use async_::AsyncTracer;
#[cfg(feature = "sync")]
pub use sync_::SyncTracer;

use rustracing::sampler::{BoxSampler, Sampler};
use rustracing::span::SpanSend;
use rustracing::Tracer as InnerTracer;

use std::fmt;

use crate::span::SpanContextState;

/// Tracer.
#[derive(Clone)]
pub struct Tracer<Sender: SpanSend<SpanContextState>> {
    inner: InnerTracer<BoxSampler<SpanContextState>, SpanContextState, Sender>,
}

impl<Sender: SpanSend<SpanContextState>> Tracer<Sender> {
    /// Makes a new `Tracer` instance.
    pub fn with_sender<S>(sampler: S, span_tx: Sender) -> Self
    where
        S: Sampler<SpanContextState> + Send + Sync + 'static,
    {
        let inner = InnerTracer::with_sender(sampler.boxed(), span_tx);
        Tracer { inner }
    }

    /// Clone with the given `sampler`.
    pub fn clone_with_sampler<T>(&self, sampler: T) -> Self
    where
        T: Sampler<SpanContextState> + Send + Sync + 'static,
    {
        let inner = self.inner.clone_with_sampler(sampler.boxed());
        Tracer { inner }
    }
}

impl<Sender: SpanSend<SpanContextState>> fmt::Debug for Tracer<Sender> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Tracer {{ .. }}")
    }
}

#[cfg(not(feature = "sync"))]
mod async_ {
    use crate::span::{SpanReceiver, SpanSender, StartSpanOptions};
    use crate::tracer::{SpanContextState, Tracer};

    use beef::lean::Cow;
    use rustracing::sampler::Sampler;

    /// An async tracer
    pub type AsyncTracer = Tracer<SpanSender>;

    impl AsyncTracer {
        /// Makes a new `Tracer` instance with an unbounded channel.
        ///
        /// This constructor is mainly for backward compatibility, it has the same interface
        /// as in previous versions except the type of `SpanReceiver`.
        /// It builds an unbounded channel which may cause memory issues if there is no reader,
        /// prefer `with_sender()` alternative with a bounded one.
        pub fn new<S>(sampler: S) -> (Self, SpanReceiver)
        where
            S: Sampler<SpanContextState> + Send + Sync + 'static,
        {
            let (inner, rx) = rustracing::Tracer::new_async(sampler.boxed());
            (Tracer { inner }, rx)
        }

        /// Returns `StartSpanOptions` for starting a span which has the name `operation_name`.
        pub fn span<N>(&self, operation_name: N) -> StartSpanOptions
        where
            N: Into<Cow<'static, str>>,
        {
            self.inner.span(operation_name)
        }
    }
}

#[cfg(feature = "sync")]
mod sync_ {
    use crate::span::{SpanReceiver, SpanSender, StartSpanOptions};
    use crate::tracer::{SpanContextState, Tracer};

    use beef::lean::Cow;
    use rustracing::sampler::Sampler;

    /// An sync tracer.
    pub type SyncTracer = Tracer<SpanSender>;

    impl SyncTracer {
        /// Makes a new `Tracer` instance with an unbounded channel.
        ///
        /// This constructor is mainly for backward compatibility, it has the same interface
        /// as in previous versions except the type of `SpanReceiver`.
        /// It builds an unbounded channel which may cause memory issues if there is no reader,
        /// prefer `with_sender()` alternative with a bounded one.
        pub fn new<S>(sampler: S) -> (Self, SpanReceiver)
        where
            S: Sampler<SpanContextState> + Send + Sync + 'static,
        {
            let (inner, rx) = rustracing::Tracer::new(sampler.boxed());
            (Tracer { inner }, rx)
        }

        /// Returns `StartSpanOptions` for starting a span which has the name `operation_name`.
        pub fn span<N>(&self, operation_name: N) -> StartSpanOptions
        where
            N: Into<Cow<'static, str>>,
        {
            self.inner.span(operation_name)
        }
    }
}

#[cfg(test)]
mod test {
    use rustracing::sampler::NullSampler;

    use super::*;

    #[test]
    fn is_tracer_sendable() {
        fn is_send<T: Send>(_: T) {}

        let (span_tx, _span_rx) = tokio::sync::mpsc::channel(10);
        let tracer = Tracer::with_sender(NullSampler, span_tx);
        is_send(tracer);
    }
}
