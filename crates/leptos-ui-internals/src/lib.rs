//! Shared internal infrastructure for the Leptos port of Base UI — the Phase A infra units of
//! `packages/react/src` (`use-render`, `merge-props`, `internals`, `floating-ui-react`,
//! `csp-provider`, `direction-provider`, `types`, `unstable-use-media-query`, `utils`), all
//! consolidated into this one crate by the crate-workspace decision in `specs/architecture.md`.

pub mod csp_context;
pub mod csp_provider;
pub mod use_media_query;

pub use csp_context::{CSPContextValue, use_csp_context};
pub use csp_provider::provide_csp_context;
pub use use_media_query::{
    MatchMediaFn, MatchMediaSource, SsrMatchMediaFn, UseMediaQueryOptions, use_media_query,
};
