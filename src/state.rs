use crate::{PeleError, RenderOptions, RenderResult};

pub fn render(_source: &str, _options: &RenderOptions) -> Result<RenderResult, PeleError> {
    Err(PeleError::unsupported("State diagrams are not ported yet"))
}
