//! Core type definitions for the Context Engine.
//!
//! This module contains all the shared types used across the codebase,
//! organized into sub-modules for different domains.

pub mod planning;
pub mod review;
pub mod search;

// Re-export commonly used types
pub use planning::*;
pub use review::*;
pub use search::*;
