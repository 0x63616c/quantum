use super::*;
use quantum_plugin_api::{
    CommentsAction, CommentsCx, CommentsOutcome, CommentsQuery, CommentsView, Value,
};

impl Host {
    /// Whether an enabled panel binds portable discussion behavior.
    pub fn has_comments_panel(&mut self, id: &str) -> Result<bool, HostError> {
        let Some(index) = self.comments_owner(id) else {
            return Ok(false);
        };
        self.invoke(index, Phase::Comments, |plugin| {
            plugin.comments_panel(id).is_some()
        })
    }

    /// Read an enabled panel's presentation data through immutable capabilities.
    pub fn comments_view(
        &mut self,
        id: &str,
        cx: &dyn CommentsCx,
    ) -> Result<Option<CommentsView>, HostError> {
        let Some(index) = self.comments_owner(id) else {
            return Ok(None);
        };
        self.invoke(index, Phase::Comments, |plugin| {
            plugin.comments_panel(id).map(|panel| panel.view(cx))
        })
    }

    /// Propose discussion edits and state without admitting any effects.
    pub fn dispatch_comments(
        &mut self,
        id: &str,
        cx: &dyn CommentsCx,
        action: CommentsAction,
    ) -> Result<Option<CommentsOutcome>, HostError> {
        let Some(index) = self.comments_owner(id) else {
            return Ok(None);
        };
        if let CommentsAction::Command(command) = &action
            && !self.plugins[index]
                .contributions
                .iter()
                .any(|c| matches!(c, Contribution::Command(spec) if spec.id == *command))
        {
            return Ok(None);
        }
        self.invoke(index, Phase::Comments, |plugin| {
            plugin
                .comments_panel(id)
                .map(|panel| panel.interact(cx, action))
        })?
        .transpose()
        .map_err(Into::into)
    }

    /// Read the existing agent pagination shape from an enabled discussion panel.
    pub fn list_comments(
        &mut self,
        id: &str,
        cx: &dyn CommentsCx,
        query: CommentsQuery,
    ) -> Result<Option<Value>, HostError> {
        let Some(index) = self.comments_owner(id) else {
            return Ok(None);
        };
        self.invoke(index, Phase::Comments, |plugin| {
            plugin.comments_panel(id).map(|panel| panel.list(cx, query))
        })
    }

    fn comments_owner(&self, id: &str) -> Option<usize> {
        self.plugins.iter().position(|plugin| {
            plugin.state == PluginState::Enabled
                && plugin
                    .contributions
                    .iter()
                    .any(|c| matches!(c, Contribution::Panel(panel) if panel.id == id))
        })
    }
}
