use crate::{PeleError, RenderOptions, RenderResult};

pub fn render(_source: &str, _options: &RenderOptions) -> Result<RenderResult, PeleError> {
    Err(PeleError::unsupported("Class diagrams are not ported yet"))
}
