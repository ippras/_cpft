use serde::{Deserialize, Serialize};

/// Windows
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Windows {
    pub open_metadata: bool,
    pub open_settings: bool,
    pub open_sum: bool,
}

impl Windows {
    pub fn new() -> Self {
        Self {
            open_metadata: false,
            open_settings: false,
            open_sum: false,
        }
    }
}
