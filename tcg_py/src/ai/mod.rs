//! AI implementations for Pokemon TCG.

pub mod traits;
pub mod random_ai_v4;
pub mod react;

pub use traits::AiController;
pub use random_ai_v4::RandomAiV4;
