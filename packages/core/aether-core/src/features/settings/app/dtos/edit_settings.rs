use serde::{Deserialize, Serialize};

use crate::features::settings::Settings;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditSettings {
    pub max_concurrent_downloads: usize,
}

impl EditSettings {
    pub fn apply_to(self, settings: &mut Settings) -> bool {
        let mut is_changed = false;

        if settings.max_concurrent_downloads() != self.max_concurrent_downloads {
            settings.set_max_concurrent_downloads(self.max_concurrent_downloads);
            is_changed = true;
        };

        is_changed
    }
}
