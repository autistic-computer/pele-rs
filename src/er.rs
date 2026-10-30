use crate::{PeleError, RenderOptions, RenderResult};

pub fn render(_source: &str, _options: &RenderOptions) -> Result<RenderResult, PeleError> {
    Err(PeleError::unsupported(
        "Entity relationship diagrams are not ported yet",
    ))
}
