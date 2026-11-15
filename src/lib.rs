//! 自作MILPソルバーの数値コア。外部の最適化ライブラリは使用しない。
pub mod lp;
pub mod model;
pub mod mps;
pub mod propagate;
pub mod revised;
pub mod solve;

pub mod heuristic;

pub mod branching;

pub(crate) mod scaling;

pub(crate) mod exact;

pub mod search;

pub mod cuts;
pub mod rins;

pub mod objective_bound;

/// 計測時に共有targetから別worktreeのcrateを拾う事故を検出する。
pub const BUILD_SOURCE_DIRECTORY: &str = env!("CARGO_MANIFEST_DIR");

/// Rustソース/Cargo設定のFNV64。計測記録のSHA256とは別のcache取り違え検出用。
pub const BUILD_SOURCE_FINGERPRINT: &str = env!("MIP_SOURCE_FINGERPRINT");

pub mod binary_graph;

pub(crate) mod certificate;
