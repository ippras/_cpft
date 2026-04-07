use serde::{Deserialize, Serialize};

/// Windows
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Windows {
    pub open_correlation: bool,
    pub open_metadata: bool,
    pub open_regression: bool,
    pub open_settings: bool,
}

impl Windows {
    pub fn new() -> Self {
        Self {
            open_correlation: false,
            open_metadata: false,
            open_regression: false,
            open_settings: false,
        }
    }
}
