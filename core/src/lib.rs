//! scan-core: secp256k1 range engine for the Bitcoin puzzle challenge.
//!
//! A range is walked with one point addition per key (P += G); the affine
//! conversion of each batch shares a single field inversion (Montgomery trick).

pub mod addr;
pub mod range;
