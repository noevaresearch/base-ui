//! Port of `packages/react/src/internals/RequestQueue.ts:1-126` — the concurrency-limited
//! fetch queue shared by the data-loading components (`TODO.md`, item `infra: internals`).
//!
//! Upstream is a single class tracking each key through `queued → pending → (removed)`
//! (`RequestQueue.ts:23-25`): two insertion-ordered maps (`queuedRequests` FIFO via `Map`
//! insertion order, `pendingRequests`, `:27-29`), `processQueue` filling spare concurrency
//! slots from `pickEntries` (`:47-55`, the documented subclass override point for ordering —
//! default FIFO, untested per the implementation spec's "Anything in source" item 8), a
//! rejected `fetchFn` caught to just delete the pending entry (`:76-80`), and the queue
//! driven re-entrantly after every settle (`:84-86`, `:99-114`).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - The insertion-ordered maps port to `Vec<(String, T)>` registries: `Map.set` on an
//!   existing key replaces the value in place (position kept), `Map.delete` removes, and
//!   iteration is insertion order — the properties `pickEntries`' FIFO contract needs.
//! - Upstream's methods are async arrow properties whose bodies run synchronously until
//!   their first `await` (`queue` enqueues and starts the first batch synchronously —
//!   `RequestQueue.ts:89-97` — which is why the status assertions after an un-awaited
//!   `queue(...)` call see `pending`/`queued`). The port mirrors that split: the public
//!   methods perform their synchronous part (enqueue / pending removal / next batch start)
//!   at call time and return a [`ProcessFuture`] the caller drives — the analog of
//!   upstream's promise, which JS runs on its microtask queue whether awaited or not. A
//!   dropped future stops the post-batch continuation, like an un-awaited promise chain a
//!   caller abandons.
//! - `processQueue`'s recursion (`:84-86`) ports to the returned future's loop: after each
//!   settled batch, start the next while slots and queued work remain; ending the loop
//!   mirrors the recursion's own early returns (`:57-68`).
//! - The rejection catch (`:77-80`) ports to the per-fetch wrapper: an `Err` output removes
//!   the pending entry; the batch (`Promise.all`, `:83` → `join_all`) then completes and the
//!   queue continues. `fetchFn`'s future output is `Result<(), `[`FetchError`]`>` — the
//!   rejection reason is as unobservable to the queue as upstream's caught error.
//! - Keys: `getKeyId ?? String` (`:40`) ports to a required key-id function; the default
//!   (`String(key)`) is [`RequestQueue::new`]'s `Display` bound, and complex keys use
//!   [`RequestQueue::with_key_id`] (the complex-key test's `getKeyId`,
//!   `RequestQueue.test.ts:127-138`). `fetchFn` receives its own clone of the key, the
//!   value-sharing analog of upstream passing the same reference to both the pending
//!   registry and the fetch (for handle types like `Rc`, identity is preserved by the
//!   clone).
//! - `maxConcurrentRequests ?? Infinity` (`:39`) ports to `usize::MAX`.
//! - `pickEntries`' protected-override point (`:43-55`) ports as the inline FIFO drain:
//!   Rust structs have no subclassing, and only the default FIFO order is exercised
//!   upstream. A future ordering policy arrives as a separate decision, not a silent
//!   rewrite.
//! - Re-entrancy: `fetchFn` is invoked outside the registry borrow, so a fetch's
//!   synchronous part may call back into the queue, as it may upstream.

use std::cell::RefCell;
use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

use futures::future::{LocalBoxFuture, join_all};

/// The upstream `RequestStatus` (`packages/react/src/internals/RequestQueue.ts:1`):
/// `'pending'` while `fetchFn` runs, `'queued'` while waiting for a concurrency slot,
/// `'unknown'` for keys never queued or no longer tracked (`RequestQueue.test.ts:26-38`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestStatus {
    /// `'pending'`.
    Pending,
    /// `'queued'`.
    Queued,
    /// `'unknown'`.
    Unknown,
}

impl RequestStatus {
    /// The upstream string literal (the `RequestQueue.test.ts:33-35,60-62` assertions).
    pub fn as_str(self) -> &'static str {
        match self {
            RequestStatus::Pending => "pending",
            RequestStatus::Queued => "queued",
            RequestStatus::Unknown => "unknown",
        }
    }
}

/// The erased `fetchFn` rejection — the queue only ever observes *that* a fetch failed
/// (`packages/react/src/internals/RequestQueue.ts:77-80` catches and discards the reason).
pub struct FetchError(pub Box<dyn std::error::Error + 'static>);

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Debug for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&*self.0, f)
    }
}

impl std::error::Error for FetchError {}

/// The erased fetch future — upstream's `fetchFn: (key) => Promise<void>`
/// (`packages/react/src/internals/RequestQueue.ts:7`).
pub type FetchFuture = Pin<Box<dyn Future<Output = Result<(), FetchError>> + 'static>>;

/// The queue's per-batch continuation — upstream's `processQueue` promise
/// (`packages/react/src/internals/RequestQueue.ts:57-87`). Local: the registry is an
/// `Rc` cell, like the JS instance.
pub type ProcessFuture = LocalBoxFuture<'static, ()>;

struct Inner<T> {
    pending_requests: Vec<(String, T)>,
    queued_requests: Vec<(String, T)>,
}

impl<T> Inner<T> {
    /// `Map.set` on an existing key replaces the value in place (position kept).
    fn set(registry: &mut Vec<(String, T)>, key_id: String, key: T) {
        if let Some(entry) = registry.iter_mut().find(|(id, _)| *id == key_id) {
            entry.1 = key;
        } else {
            registry.push((key_id, key));
        }
    }

    fn has(registry: &[(String, T)], key_id: &str) -> bool {
        registry.iter().any(|(id, _)| id == key_id)
    }

    fn remove(registry: &mut Vec<(String, T)>, key_id: &str) {
        registry.retain(|(id, _)| id != key_id);
    }
}

/// The queue's shared internals — upstream's instance fields
/// (`packages/react/src/internals/RequestQueue.ts:27-40`). Cloned into every
/// [`ProcessFuture`] so the continuation stays `'static`, the way a JS promise closure
/// holds its instance.
struct Shared<T> {
    inner: RefCell<Inner<T>>,
    fetch_fn: Rc<dyn Fn(T) -> FetchFuture>,
    /// `maxConcurrentRequests` — a `Cell` so the chained
    /// [`RequestQueue::max_concurrent_requests`] setter can adjust it through the shared
    /// handle (`usize::MAX` is the `Infinity` default).
    max_concurrent_requests: std::cell::Cell<usize>,
    get_key_id: Rc<dyn Fn(&T) -> String>,
}

/// The upstream `RequestQueue` (`packages/react/src/internals/RequestQueue.ts:26-126`).
/// Clone handles share one queue, the way JS callers share the instance.
#[derive(Clone)]
pub struct RequestQueue<T> {
    shared: Rc<Shared<T>>,
}

impl<T: Clone + 'static> RequestQueue<T> {
    /// The constructor with the default `String(key)` key id
    /// (`packages/react/src/internals/RequestQueue.ts:40`), requiring `Display` of the key.
    pub fn new(fetch_fn: impl Fn(T) -> FetchFuture + 'static) -> Self
    where
        T: Display,
    {
        Self::with_key_id(fetch_fn, |key| key.to_string())
    }

    /// The constructor with a custom `getKeyId`
    /// (`packages/react/src/internals/RequestQueue.ts:17-18`), required for complex key
    /// types.
    pub fn with_key_id(
        fetch_fn: impl Fn(T) -> FetchFuture + 'static,
        get_key_id: impl Fn(&T) -> String + 'static,
    ) -> Self {
        Self {
            shared: Rc::new(Shared {
                inner: RefCell::new(Inner {
                    pending_requests: Vec::new(),
                    queued_requests: Vec::new(),
                }),
                fetch_fn: Rc::new(fetch_fn),
                max_concurrent_requests: std::cell::Cell::new(usize::MAX),
                get_key_id: Rc::new(get_key_id),
            }),
        }
    }

    /// The `maxConcurrentRequests` option
    /// (`packages/react/src/internals/RequestQueue.ts:10-12`), chained after the
    /// constructor; `usize::MAX` is the `Infinity` default.
    pub fn max_concurrent_requests(self, max: usize) -> Self {
        self.shared.max_concurrent_requests.set(max);
        self
    }

    /// The upstream `queue` (`packages/react/src/internals/RequestQueue.ts:89-97`):
    /// enqueues every key not already pending — synchronously, like upstream's pre-await
    /// body — starts the first batch (synchronously, like upstream's pre-await
    /// `processQueue` head, `:57-81`), and returns the processing continuation. A key
    /// already queued replaces its entry in place (`Map.set` semantics).
    pub fn queue(&self, keys: impl IntoIterator<Item = T>) -> ProcessFuture {
        {
            let mut inner = self.shared.inner.borrow_mut();
            for key in keys {
                let key_id = (self.shared.get_key_id)(&key);
                // Upstream re-queues only when the key is not pending
                // (`RequestQueue.ts:92`).
                if !Inner::has(&inner.pending_requests, &key_id) {
                    Inner::set(&mut inner.queued_requests, key_id, key);
                }
            }
        }
        process_queue_with_first_batch(Rc::clone(&self.shared), Self::start_batch(&self.shared))
    }

    /// The upstream `setRequestSettled` (`packages/react/src/internals/RequestQueue.ts:99-103`):
    /// drops the pending entry (synchronously), starts the next batch (synchronously, like
    /// upstream's pre-await `processQueue` head), and returns the continuation.
    pub fn set_request_settled(&self, key: &T) -> ProcessFuture {
        let key_id = (self.shared.get_key_id)(key);
        Inner::remove(
            &mut self.shared.inner.borrow_mut().pending_requests,
            &key_id,
        );
        process_queue_with_first_batch(Rc::clone(&self.shared), Self::start_batch(&self.shared))
    }

    /// The upstream `clearPendingRequest`
    /// (`packages/react/src/internals/RequestQueue.ts:110-114`): identical to
    /// [`RequestQueue::set_request_settled`] upstream and here.
    pub fn clear_pending_request(&self, key: &T) -> ProcessFuture {
        self.set_request_settled(key)
    }

    /// The upstream `clear` (`packages/react/src/internals/RequestQueue.ts:105-108`):
    /// resets everything, synchronously.
    pub fn clear(&self) {
        let mut inner = self.shared.inner.borrow_mut();
        inner.queued_requests.clear();
        inner.pending_requests.clear();
    }

    /// The upstream `getRequestStatus` (`packages/react/src/internals/RequestQueue.ts:116-125`).
    pub fn get_request_status(&self, key: &T) -> RequestStatus {
        let key_id = (self.shared.get_key_id)(key);
        let inner = self.shared.inner.borrow();
        if Inner::has(&inner.pending_requests, &key_id) {
            return RequestStatus::Pending;
        }
        if Inner::has(&inner.queued_requests, &key_id) {
            return RequestStatus::Queued;
        }
        RequestStatus::Unknown
    }

    /// The upstream `processQueue`'s synchronous half
    /// (`packages/react/src/internals/RequestQueue.ts:57-81`): the slot guard, then moving
    /// the next FIFO batch (`pickEntries`, `:47-55`) from queued to pending and
    /// constructing its fetch futures. An empty result means the continuation has nothing
    /// to await — the recursion's early returns (`:57-68`).
    fn start_batch(shared: &Rc<Shared<T>>) -> Vec<FetchFuture> {
        // The batch leaves the queued registry and enters the pending registry before any
        // fetch future is constructed (`RequestQueue.ts:72-74`); `fetch_fn` runs outside
        // the borrow so its synchronous part can re-enter the queue.
        let batch: Vec<(String, T)> = {
            let mut inner = shared.inner.borrow_mut();
            if inner.queued_requests.is_empty()
                || inner.pending_requests.len() >= shared.max_concurrent_requests.get()
            {
                return Vec::new();
            }
            let loop_length = (shared.max_concurrent_requests.get() - inner.pending_requests.len())
                .min(inner.queued_requests.len());
            if loop_length == 0 {
                return Vec::new();
            }
            let batch: Vec<(String, T)> = inner.queued_requests.drain(..loop_length).collect();
            for (key_id, key) in &batch {
                Inner::set(&mut inner.pending_requests, key_id.clone(), key.clone());
            }
            batch
        };

        batch
            .into_iter()
            .map(|(key_id, key)| {
                let fetch = (shared.fetch_fn)(key);
                let shared = Rc::clone(shared);
                let continuation: FetchFuture = Box::pin(async move {
                    if fetch.await.is_err() {
                        // The catch handler: a rejected fetch deletes its pending entry
                        // (`packages/react/src/internals/RequestQueue.ts:77-80`).
                        Inner::remove(&mut shared.inner.borrow_mut().pending_requests, &key_id);
                    }
                    Ok::<(), FetchError>(())
                });
                continuation
            })
            .collect()
    }
}

/// The upstream `processQueue` body
/// (`packages/react/src/internals/RequestQueue.ts:57-87`) as a driven future: await the
/// already-started batch (`Promise.all`), then start the next while the queue has work and
/// slots (the `:84-86` recursion).
fn process_queue_with_first_batch<T: Clone + 'static>(
    shared: Rc<Shared<T>>,
    first_batch: Vec<FetchFuture>,
) -> ProcessFuture {
    Box::pin(async move {
        if !first_batch.is_empty() {
            join_all(first_batch).await;
        }
        loop {
            let batch = RequestQueue::<T>::start_batch(&shared);
            if batch.is_empty() {
                break;
            }
            join_all(batch).await;
        }
    })
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;
    use std::task::{Context, Poll, Waker};

    use any_spawner::Executor;
    use futures::executor::block_on;

    use super::*;

    fn init_executor() {
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| {
            Executor::init_futures_executor();
        });
    }

    /// The test-side analog of upstream's `createDeferred`
    /// (`packages/react/src/internals/RequestQueue.test.ts:4-12`): a manually-resolvable
    /// fetch future.
    #[derive(Clone)]
    struct Deferred {
        state: Rc<RefCell<DeferredState>>,
    }

    enum DeferredState {
        Unsettled(Option<Waker>),
        Done(Option<Result<(), FetchError>>),
    }

    impl Deferred {
        fn new() -> Deferred {
            Deferred {
                state: Rc::new(RefCell::new(DeferredState::Unsettled(None))),
            }
        }

        fn settle(&self, result: Result<(), FetchError>) {
            let waker = if let DeferredState::Unsettled(waker) = &mut *self.state.borrow_mut() {
                waker.take()
            } else {
                return;
            };
            *self.state.borrow_mut() = DeferredState::Done(Some(result));
            if let Some(waker) = waker {
                waker.wake();
            }
        }

        fn resolve(&self) {
            self.settle(Ok(()));
        }

        fn reject(&self) {
            self.settle(Err(FetchError("rejected".to_string().into())));
        }
    }

    impl Future for Deferred {
        type Output = Result<(), FetchError>;

        fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let mut state = self.state.borrow_mut();
            match &mut *state {
                DeferredState::Unsettled(waker) => {
                    *waker = Some(cx.waker().clone());
                    Poll::Pending
                }
                DeferredState::Done(result) => {
                    let result = result.take().expect("settled once");
                    Poll::Ready(result)
                }
            }
        }
    }

    fn deferred_fetch<T: Clone + 'static>() -> (
        impl Fn(T) -> FetchFuture + 'static,
        Rc<RefCell<HashMap<String, Deferred>>>,
    )
    where
        T: std::fmt::Display,
    {
        let deferreds: Rc<RefCell<HashMap<String, Deferred>>> =
            Rc::new(RefCell::new(HashMap::new()));
        let deferreds_for_fetch = Rc::clone(&deferreds);
        let fetch = move |key: T| {
            let deferred = Deferred::new();
            deferreds_for_fetch
                .borrow_mut()
                .insert(key.to_string(), deferred.clone());
            Box::pin(async move { deferred.await }) as FetchFuture
        };
        (fetch, deferreds)
    }

    fn recorded_fetch<T: Clone + 'static>()
    -> (impl Fn(T) -> FetchFuture + 'static, Rc<RefCell<Vec<T>>>) {
        let calls: Rc<RefCell<Vec<T>>> = Rc::new(RefCell::new(Vec::new()));
        let calls_for_fetch = Rc::clone(&calls);
        let fetch = move |key: T| {
            calls_for_fetch.borrow_mut().push(key.clone());
            Box::pin(async { Ok(()) }) as FetchFuture
        };
        (fetch, calls)
    }

    /// Drives the spawned queue continuation until the flag flips — the `await
    /// vi.waitFor(...)` analog (`packages/react/src/internals/RequestQueue.test.ts:118-121`).
    fn pump_until(done: &Cell<bool>) {
        for _ in 0..100 {
            Executor::poll_local();
            if done.get() {
                return;
            }
        }
        panic!("the queue continuation never completed");
    }

    // Mirrors `packages/react/src/internals/RequestQueue.test.ts:15-24`.
    #[test]
    fn calls_fetch_fn_for_each_queued_key() {
        let (fetch, calls) = recorded_fetch();
        let queue = RequestQueue::new(fetch);

        block_on(queue.queue(["a".to_string(), "b".to_string()]));

        let calls = calls.borrow();
        assert_eq!(calls.len(), 2);
        assert!(calls.contains(&"a".to_string()));
        assert!(calls.contains(&"b".to_string()));
    }

    // Mirrors `packages/react/src/internals/RequestQueue.test.ts:26-38`: the status checks
    // run against the synchronous queue head, before any continuation is driven.
    #[test]
    fn reports_correct_request_status() {
        let (fetch, _deferreds) = deferred_fetch();
        let queue = RequestQueue::new(fetch).max_concurrent_requests(1);

        let _continuation = queue.queue(["a".to_string(), "b".to_string()]);

        assert_eq!(
            queue.get_request_status(&"a".to_string()).as_str(),
            "pending"
        );
        assert_eq!(
            queue.get_request_status(&"b".to_string()).as_str(),
            "queued"
        );
        assert_eq!(
            queue.get_request_status(&"c".to_string()).as_str(),
            "unknown"
        );
    }

    // Mirrors `packages/react/src/internals/RequestQueue.test.ts:40-51`.
    #[test]
    fn does_not_re_queue_a_key_that_is_already_pending() {
        let (fetch, calls) = recorded_fetch();
        let queue = RequestQueue::new(fetch);

        let _first = queue.queue(["a".to_string()]);
        let _second = queue.queue(["a".to_string()]);

        assert_eq!(calls.borrow().len(), 1);
    }

    // Mirrors `packages/react/src/internals/RequestQueue.test.ts:53-63`.
    #[test]
    fn respects_max_concurrent_requests() {
        let (fetch, _deferreds) = deferred_fetch();
        let queue = RequestQueue::new(fetch).max_concurrent_requests(2);

        let _continuation = queue.queue(["a".to_string(), "b".to_string(), "c".to_string()]);

        assert_eq!(
            queue.get_request_status(&"a".to_string()).as_str(),
            "pending"
        );
        assert_eq!(
            queue.get_request_status(&"b".to_string()).as_str(),
            "pending"
        );
        assert_eq!(
            queue.get_request_status(&"c".to_string()).as_str(),
            "queued"
        );
    }

    // Mirrors `packages/react/src/internals/RequestQueue.test.ts:65-80`.
    #[test]
    fn processes_next_queued_item_on_set_request_settled() {
        let (fetch, _deferreds) = deferred_fetch();
        let queue = RequestQueue::new(fetch).max_concurrent_requests(1);

        let _continuation = queue.queue(["a".to_string(), "b".to_string()]);
        assert_eq!(
            queue.get_request_status(&"a".to_string()).as_str(),
            "pending"
        );
        assert_eq!(
            queue.get_request_status(&"b".to_string()).as_str(),
            "queued"
        );

        // Don't drive the continuation — like upstream's un-awaited call, the queue head
        // already started 'b' (`RequestQueue.test.ts:74-76`).
        let _settled = queue.set_request_settled(&"a".to_string());

        assert_eq!(
            queue.get_request_status(&"a".to_string()).as_str(),
            "unknown"
        );
        assert_eq!(
            queue.get_request_status(&"b".to_string()).as_str(),
            "pending"
        );
    }

    // Mirrors `packages/react/src/internals/RequestQueue.test.ts:82-92`.
    #[test]
    fn processes_next_queued_item_on_clear_pending_request() {
        let (fetch, _deferreds) = deferred_fetch();
        let queue = RequestQueue::new(fetch).max_concurrent_requests(1);

        let _continuation = queue.queue(["a".to_string(), "b".to_string()]);

        let _cleared = queue.clear_pending_request(&"a".to_string());

        assert_eq!(
            queue.get_request_status(&"a".to_string()).as_str(),
            "unknown"
        );
        assert_eq!(
            queue.get_request_status(&"b".to_string()).as_str(),
            "pending"
        );
    }

    // Mirrors `packages/react/src/internals/RequestQueue.test.ts:94-103`.
    #[test]
    fn clears_all_queued_and_pending_requests() {
        let (fetch, _deferreds) = deferred_fetch();
        let queue = RequestQueue::new(fetch).max_concurrent_requests(1);

        let _continuation = queue.queue(["a".to_string(), "b".to_string()]);
        queue.clear();

        assert_eq!(
            queue.get_request_status(&"a".to_string()).as_str(),
            "unknown"
        );
        assert_eq!(
            queue.get_request_status(&"b".to_string()).as_str(),
            "unknown"
        );
    }

    // Mirrors `packages/react/src/internals/RequestQueue.test.ts:105-125`.
    #[test]
    fn continues_processing_when_fetch_fn_rejects() {
        init_executor();
        let (fetch, deferreds) = deferred_fetch();
        let queue = RequestQueue::new(fetch).max_concurrent_requests(1);

        let done = Rc::new(Cell::new(false));
        let continuation = queue.queue(["a".to_string(), "b".to_string()]);
        let done_flag = Rc::clone(&done);
        Executor::spawn_local(async move {
            continuation.await;
            done_flag.set(true);
        });

        // The first poll starts 'a'.
        Executor::poll_local();

        // Reject 'a' → removed from pending by the catch handler, the queue continues to 'b'
        // (`RequestQueue.test.ts:116-121`).
        deferreds.borrow().get("a").expect("a deferred").reject();
        for _ in 0..100 {
            Executor::poll_local();
            if queue.get_request_status(&"a".to_string()) == RequestStatus::Unknown
                && queue.get_request_status(&"b".to_string()) == RequestStatus::Pending
            {
                break;
            }
        }
        assert_eq!(
            queue.get_request_status(&"a".to_string()).as_str(),
            "unknown"
        );
        assert_eq!(
            queue.get_request_status(&"b".to_string()).as_str(),
            "pending"
        );

        deferreds.borrow().get("b").expect("b deferred").resolve();
        pump_until(&done);
    }

    // Mirrors `packages/react/src/internals/RequestQueue.test.ts:127-138`.
    #[test]
    fn supports_custom_get_key_id_for_complex_key_types() {
        #[derive(Debug, Clone)]
        struct Key {
            id: u32,
        }

        let (fetch, calls) = recorded_fetch();
        let queue = RequestQueue::with_key_id(fetch, |key: &Key| key.id.to_string());

        block_on(queue.queue([Key { id: 1 }, Key { id: 2 }, Key { id: 1 }]));

        // { id: 1 } appears twice but should be deduplicated
        // (`RequestQueue.test.ts:136-137`).
        assert_eq!(calls.borrow().len(), 2);
    }

    // Mirrors `packages/react/src/internals/RequestQueue.test.ts:140-151`.
    #[test]
    fn processes_keys_in_fifo_order() {
        let (fetch, calls) = recorded_fetch();
        let queue = RequestQueue::new(fetch);

        block_on(queue.queue(["c".to_string(), "a".to_string(), "b".to_string()]));

        assert_eq!(
            *calls.borrow(),
            vec!["c".to_string(), "a".to_string(), "b".to_string()]
        );
    }
}
