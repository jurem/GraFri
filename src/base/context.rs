//! [`Context`]: what a term is read under.

use std::any::Any;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::random::{Xoshiro256StarStar, stream_seed};

/// What a term or a labeling is read under: the directedness, the master seed of the random
/// parts, and a cache of what they generated. Clones share the cache. A
/// [`GraphOf`](crate::GraphOf) starts with an empty cache of its own, so the cache lives and dies
/// with the graph. The cache only saves work: it never changes a result.
#[derive(Debug, Clone, Default)]
pub struct Context {
    /// Whether the edges are directed. Default: `false`.
    pub directed: bool,
    /// The master seed of the random parts. Default: 0.
    pub seed: u64,
    cache: Arc<Mutex<BTreeMap<CacheKey, Arc<dyn Any + Send + Sync>>>>,
}

/// A leaf kind, its parameters, and the directedness and seed.
type CacheKey = (&'static str, [u64; 3], bool, u64);

/// A context from a reference to one, sharing the cache.
impl From<&Context> for Context {
    fn from(context: &Context) -> Context {
        context.clone()
    }
}

impl Context {
    /// A context for the given directedness, with seed 0.
    pub fn new(directed: bool) -> Context {
        Context {
            directed,
            ..Context::default()
        }
    }

    /// A context for directed graphs, with seed 0.
    pub fn directed() -> Context {
        Context::new(true)
    }

    /// A context for undirected graphs, with seed 0; also the default.
    pub fn undirected() -> Context {
        Context::new(false)
    }

    /// The same context with an empty cache of its own.
    pub(crate) fn with_fresh_cache(self) -> Context {
        Context {
            cache: Arc::default(),
            ..self
        }
    }

    /// The same context with another master seed. The cache is shared.
    pub fn with_seed(self, seed: u64) -> Context {
        Context { seed, ..self }
    }

    /// A fresh generator for the random stream number `stream`, seeded with
    /// [`stream_seed`]`(self.seed, stream)`.
    pub fn rng(&self, stream: u64) -> Xoshiro256StarStar {
        Xoshiro256StarStar::from_seed(stream_seed(self.seed, stream))
    }

    /// The value of the leaf `leaf` with the parameters `params`, made by `make` the first time and
    /// then kept. `make` must be a pure function of them and the context, and a leaf must always
    /// cache the same type; on a type clash the second value is made again and not kept.
    pub(crate) fn cached<T: Any + Send + Sync>(
        &self,
        leaf: &'static str,
        params: [u64; 3],
        make: impl FnOnce() -> T,
    ) -> Arc<T> {
        let key = (leaf, params, self.directed, self.seed);
        let lock = || self.cache.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(value) = lock().get(&key)
            && let Ok(value) = Arc::clone(value).downcast::<T>()
        {
            return value;
        }
        // Made outside the lock; a concurrent duplicate gives the same value.
        let value = Arc::new(make());
        let erased: Arc<dyn Any + Send + Sync> = Arc::clone(&value) as _;
        let stored = Arc::clone(lock().entry(key).or_insert(erased));
        stored.downcast::<T>().unwrap_or(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::Edge;
    use std::cell::Cell;

    fn edges() -> Vec<Edge> {
        vec![Edge::undirected(0, 1)]
    }

    #[test]
    fn a_reference_converts_into_a_context_that_shares_the_cache() {
        let ctx = Context::undirected().with_seed(4);
        let copy: Context = (&ctx).into();
        assert_eq!((copy.directed, copy.seed), (false, 4));
        let a = ctx.cached("leaf", [0; 3], edges);
        let b = copy.cached("leaf", [0; 3], || unreachable!());
        assert!(Arc::ptr_eq(&a, &b));
    }

    #[test]
    fn rng_is_seeded_by_the_master_seed_and_the_stream() {
        use crate::random::Prng;
        let ctx = Context::undirected().with_seed(9);
        let draw = |ctx: &Context, stream| ctx.rng(stream).next_u64();
        assert_eq!(
            draw(&ctx, 3),
            Xoshiro256StarStar::from_seed(stream_seed(9, 3)).next_u64()
        );
        // A fresh generator each time, the same for the same stream.
        assert_eq!(draw(&ctx, 3), draw(&ctx, 3));
        assert_ne!(draw(&ctx, 3), draw(&ctx, 4));
        assert_ne!(draw(&ctx, 3), draw(&ctx.clone().with_seed(10), 3));
    }

    #[test]
    fn cached_makes_once_per_key() {
        let ctx = Context::undirected();
        let made = Cell::new(0);
        let make = || {
            made.set(made.get() + 1);
            edges()
        };
        let a = ctx.cached("leaf", [1, 2, 3], make);
        let b = ctx.cached("leaf", [1, 2, 3], make);
        assert_eq!(made.get(), 1);
        assert!(Arc::ptr_eq(&a, &b));
        ctx.cached("leaf", [1, 2, 4], make);
        ctx.cached("other", [1, 2, 3], make);
        assert_eq!(made.get(), 3);
    }

    #[test]
    fn the_key_includes_directedness_and_seed() {
        let ctx = Context::undirected();
        let made = Cell::new(0);
        let make = || {
            made.set(made.get() + 1);
            edges()
        };
        ctx.cached("leaf", [0; 3], make);
        Context {
            directed: true,
            ..ctx.clone()
        }
        .cached("leaf", [0; 3], make);
        ctx.clone().with_seed(1).cached("leaf", [0; 3], make);
        assert_eq!(made.get(), 3);
    }

    #[test]
    fn values_of_any_type_are_cached_and_a_type_clash_is_made_again() {
        let ctx = Context::undirected();
        let a = ctx.cached("leaf", [0; 3], || String::from("kept"));
        let b = ctx.cached("leaf", [0; 3], || unreachable!());
        assert!(Arc::ptr_eq(&a, &b));
        // The same key with another type: nothing to reuse, and nothing panics.
        let n = ctx.cached("leaf", [0; 3], || 7u64);
        assert_eq!(*n, 7);
        assert_eq!(*ctx.cached("leaf", [0; 3], String::new), "kept");
    }

    #[test]
    fn clones_share_the_cache() {
        let ctx = Context::undirected();
        let a = ctx.cached("leaf", [0; 3], edges);
        let b = ctx.clone().cached("leaf", [0; 3], || unreachable!());
        assert!(Arc::ptr_eq(&a, &b));
    }
}
