//! # Sangha — Sociology Engine
//!
//! **सङ्घ** (Sanskrit: community, assembly)
//!
//! A Rust library for computational sociology: social networks, game theory,
//! group dynamics, population models, opinion dynamics, and inequality.
//!
//! ## Modules
//!
//! - [`network`] — Social network graphs, Watts-Strogatz, clustering
//! - [`game_theory`] — Nash equilibria, prisoner's dilemma, iterated games
//! - [`opinion`] — Bounded confidence, echo chambers, consensus
//! - [`group`] — Tuckman stages, social loafing, groupthink
//! - [`population`] — Logistic growth, SIR model, herd immunity
//! - [`influence`] — Conformity, social proof, Bass diffusion
//! - [`inequality`] — Gini coefficient, Lorenz curve
//!
//! ## Example
//!
//! ```
//! use sangha::population;
//!
//! // Herd immunity threshold for R0 = 3
//! let h = population::herd_immunity_threshold(3.0).unwrap();
//! assert!((h - 2.0 / 3.0).abs() < 1e-10); // ~66.7%
//!
//! // Gini coefficient of equal distribution
//! let g = sangha::inequality::gini_coefficient(&[100.0, 100.0, 100.0]).unwrap();
//! assert!(g.abs() < 1e-10); // perfect equality
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

extern crate alloc;

pub mod error;
pub mod game_theory;
pub mod group;
pub mod inequality;
pub mod influence;
pub mod network;
pub mod opinion;
pub mod population;

pub use error::SanghaError;
