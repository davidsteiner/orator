/// Configuration for code generation.
#[derive(Debug, Clone)]
pub struct Config {
    pub header_params: bool,
    pub cookie_params: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            header_params: true,
            cookie_params: true,
        }
    }
}

impl Config {
    pub fn header_params(mut self, enabled: bool) -> Self {
        self.header_params = enabled;
        self
    }

    pub fn cookie_params(mut self, enabled: bool) -> Self {
        self.cookie_params = enabled;
        self
    }

    pub fn is_location_enabled(&self, location: &crate::ir::ParamLocation) -> bool {
        match location {
            crate::ir::ParamLocation::Path | crate::ir::ParamLocation::Query => true,
            crate::ir::ParamLocation::Header => self.header_params,
            crate::ir::ParamLocation::Cookie => self.cookie_params,
        }
    }
}
