use std::sync::Arc;

use super::{ListGameLogSourcesUseCasePort, ReadGameLogUseCasePort};

/// Extension trait providing access to the game log use cases.
pub trait GameLogFeature {
    fn list_game_log_sources_use_case(&self) -> Arc<dyn ListGameLogSourcesUseCasePort>;
    fn read_game_log_use_case(&self) -> Arc<dyn ReadGameLogUseCasePort>;
}
