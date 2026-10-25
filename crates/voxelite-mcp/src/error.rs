use serde::Serialize;

pub(crate) type Result<T> = std::result::Result<T, Failure>;

#[derive(Debug, Serialize)]
pub(crate) struct Failure {
    pub code: &'static str,
    pub message: Box<str>,
    pub retryable: bool,
    pub recovery: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_revision: Option<String>,
}
impl Failure {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        let (retryable, recovery) = match code {
            "model_busy" => (true, "Retry the identical request with the original operation_id after pending requests finish."),
            "cancelled" => (true, "This request was cancelled before execution; retry the identical request with the original operation_id when ready."),
            "outcome_unknown" => (false, "Inspect the destination file and current document state before taking further action; do not blindly retry or assign a new operation_id."),
            "operation_id_reused" => (false, "Recover the original request for this operation_id. Use a fresh ID only for a new operation."),
            "epoch_conflict" | "revision_conflict" | "history_conflict" => (false, "Read current document and history state, rebase the intended change, then use a fresh operation_id."),
            _ => (false, "Inspect voxelite_document_info and the tool schema, correct the request, then use a new operation_id."),
        };
        Self {
            code,
            message: bounded(message.into(), 2048).into_boxed_str(),
            retryable,
            recovery,
            path: None,
            command_index: None,
            current_revision: None,
        }
    }
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new("invalid_argument", message)
    }
    pub fn at(mut self, path: impl Into<String>) -> Self {
        self.path = Some(bounded(path.into(), 512));
        self
    }
}
impl From<voxelite_core::Error> for Failure {
    fn from(e: voxelite_core::Error) -> Self {
        use voxelite_core::Error;
        if let Error::Batch { index, source } = e {
            let mut f = Self::from(*source);
            f.command_index = Some(index);
            f.path = Some(format!("/operations/{index}{}", f.path.unwrap_or_default()));
            return f;
        }
        let code = match &e {
            Error::Invalid(_) => "invalid_argument",
            Error::NotFound(_) => "not_found",
            Error::OutOfBounds(_) => "out_of_bounds",
            Error::LayerBlocked(_) => "layer_blocked",
            Error::Limit(_) => "limit_exceeded",
            Error::Conflict(_) => "history_conflict",
            Error::Batch { .. } => unreachable!(),
        };
        Self::new(code, e.to_string())
    }
}
impl From<std::io::Error> for Failure {
    fn from(e: std::io::Error) -> Self {
        Self::new("storage_unavailable", e.to_string())
    }
}
impl From<voxelite_format::FormatError> for Failure {
    fn from(e: voxelite_format::FormatError) -> Self {
        match e {
            voxelite_format::FormatError::Io(e) => e.into(),
            e => Self::new("invalid_format", e.to_string()),
        }
    }
}
impl From<voxelite_render::RenderError> for Failure {
    fn from(e: voxelite_render::RenderError) -> Self {
        Self::new("render_failed", e.to_string())
    }
}
impl From<voxelite_export::ExportError> for Failure {
    fn from(e: voxelite_export::ExportError) -> Self {
        Self::new("export_failed", e.to_string())
    }
}

pub(crate) struct Args<'a>(pub &'a serde_json::Value);
impl Args<'_> {
    pub fn get<T: serde::de::DeserializeOwned>(&self, name: &str) -> Result<T> {
        serde_json::from_value(self.0.get(name).cloned().unwrap_or(serde_json::Value::Null))
            .map_err(|e| Failure::invalid(e.to_string()).at(format!("/{name}")))
    }
    pub fn optional<T: serde::de::DeserializeOwned>(&self, name: &str) -> Result<Option<T>> {
        if self.0.get(name).is_none() {
            Ok(None)
        } else {
            self.get(name).map(Some)
        }
    }
    pub fn default<T: serde::de::DeserializeOwned>(&self, name: &str, value: T) -> Result<T> {
        Ok(self.optional(name)?.unwrap_or(value))
    }
}

fn bounded(mut value: String, max: usize) -> String {
    if value.len() > max {
        let mut end = max;
        while !value.is_char_boundary(end) {
            end -= 1;
        }
        value.truncate(end);
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn errors_bound_untrusted_unicode_without_splitting_characters() {
        let error = Failure::invalid("🦀".repeat(2000)).at("🦀".repeat(500));
        assert!(error.message.len() <= 2048);
        assert!(error.path.unwrap().len() <= 512);
    }
}
