use super::*;

pub(super) fn matches(comment: &Comment, filter: usize) -> bool {
    match filter {
        0 => !comment.resolved,
        1 => comment.resolved,
        _ => true,
    }
}

fn create(
    cx: &dyn CommentsCx,
    id: Uuid,
    anchor: Anchor,
    layer: CommentLayer,
    message: Uuid,
    text: String,
) -> Command {
    let layer = match layer {
        CommentLayer::Explicit(layer) => layer,
        CommentLayer::Automatic => match &anchor {
            Anchor::Voxel { at } => cx.snapshot().sample(*at).map(|(_, id)| id),
            Anchor::Region { .. } => None,
        },
    };
    Command::CreateComment {
        id,
        anchor,
        layer,
        message,
        text,
    }
}

impl CommentsPanel for CommentsPlugin {
    fn view(&self, cx: &dyn CommentsCx) -> CommentsView {
        let mut state = cx.state().clone();
        for comment in cx
            .live_snapshot()
            .comments
            .iter()
            .chain(cx.snapshot().comments.iter())
        {
            let next = state.numbers.len() + 1;
            state.numbers.entry(comment.id).or_insert(next);
        }
        let mut comments: Vec<_> = cx
            .snapshot()
            .comments
            .iter()
            .filter(|c| matches(c, state.filter))
            .cloned()
            .collect();
        comments.sort_by_key(|c| state.numbers.get(&c.id).copied().unwrap_or(0));
        let thread = state
            .selected
            .and_then(|id| {
                let comments = &cx.snapshot().comments;
                let key = comments.key(id)?;
                Some((comments.position(key)?, comments.get(key)?))
            })
            .map(|(i, c)| (i, c.clone()));
        let region = state
            .draft
            .as_ref()
            .or_else(|| thread.as_ref().map(|(_, c)| &c.anchor))
            .and_then(|a| a.region().ok());
        CommentsView {
            state,
            comments,
            empty: cx.snapshot().comments.is_empty(),
            thread,
            region,
        }
    }

    fn interact(
        &self,
        cx: &dyn CommentsCx,
        action: CommentsAction,
    ) -> Result<CommentsOutcome, ToolError> {
        let mut out = CommentsOutcome {
            state: self.view(cx).state,
            ..Default::default()
        };
        match action {
            CommentsAction::Command(id) if id == COMMAND_ID => {
                if cx.preview() {
                    return Err(ToolError::Gesture("Return to live to add a comment".into()));
                }
                out.open_panel = true;
                out.state.placing = true;
                out.state.first = None;
                out.state.selected = None;
                out.state.draft = None;
                out.status = Some(
                    if out.state.region {
                        "Click two corners to anchor a region"
                    } else {
                        "Click a voxel to anchor a comment"
                    }
                    .into(),
                );
            }
            CommentsAction::Command(_) => {
                return Err(ToolError::Gesture("Unknown comments command".into()));
            }
            CommentsAction::Anchor(at) => {
                if cx.preview() {
                    return Err(ToolError::Gesture("Return to live to add a comment".into()));
                }
                let anchor = if out.state.region {
                    if let Some(first) = out.state.first.take() {
                        Anchor::Region {
                            min: std::array::from_fn(|a| first[a].min(at[a])),
                            max: std::array::from_fn(|a| first[a].max(at[a]) + 1),
                        }
                    } else {
                        out.state.first = Some(at);
                        out.status = Some("Click the opposite corner".into());
                        return Ok(out);
                    }
                } else {
                    Anchor::Voxel { at }
                };
                out.state.draft = Some(anchor);
                out.state.placing = false;
                out.clear_text = true;
            }
            CommentsAction::Select(id) => {
                out.state.selected = id;
                out.state.draft = None;
                out.state.placing = false;
            }
            CommentsAction::Cancel => {
                out.state.placing = false;
                out.state.draft = None;
                out.state.selected = None;
            }
            CommentsAction::ToggleRegion => {
                out.state.region = !out.state.region;
                out.state.first = None;
            }
            CommentsAction::Filter(filter) => out.state.filter = filter,
            CommentsAction::Post(text) => {
                let text = text.trim().to_string();
                if let Some(anchor) = out.state.draft.clone() {
                    let id = Uuid::new_v4();
                    out.commands.push(create(
                        cx,
                        id,
                        anchor,
                        CommentLayer::Automatic,
                        Uuid::new_v4(),
                        text,
                    ));
                    out.label = "Add comment".into();
                    out.state.selected = Some(id);
                    out.state.draft = None;
                } else if let Some(id) = out.state.selected {
                    out.commands.push(Command::Reply {
                        id,
                        message: Uuid::new_v4(),
                        text,
                    });
                    out.label = "Reply to comment".into();
                } else {
                    return Ok(out);
                }
                out.clear_text = true;
                out.scroll_bottom = true;
            }
            CommentsAction::Create {
                id,
                anchor,
                layer,
                message,
                text,
            } => {
                out.commands
                    .push(create(cx, id, anchor, layer, message, text));
                out.label = "Add comment".into();
            }
            CommentsAction::Reply { id, message, text } => {
                out.commands.push(Command::Reply { id, message, text });
                out.label = "Reply to comment".into();
            }
            CommentsAction::Reanchor { id, anchor, layer } => {
                out.commands.push(Command::Reanchor { id, anchor, layer });
                out.label = "Move comment".into();
            }
            CommentsAction::Resolve { id, resolved } => {
                out.commands.push(Command::Resolve { id, resolved });
                out.label = if resolved {
                    "Resolve comment"
                } else {
                    "Reopen comment"
                }
                .into();
            }
        }
        Ok(out)
    }

    fn list(&self, cx: &dyn CommentsCx, query: CommentsQuery) -> Value {
        let s = cx.snapshot();
        let comments: Vec<_> = s
            .comments
            .iter()
            .filter(|c| query.state == "all" || c.resolved == (query.state == "resolved"))
            .collect();
        let page: Vec<_> = comments.iter().skip(query.offset).take(query.limit).map(|c| {
            let messages: Vec<_> = c.messages.iter().skip(query.message_offset).take(query.message_limit)
                .map(|m| json::json!({"id":m.id,"origin":m.origin,"text":m.text,"tool":m.tool})).collect();
            let next_message = (query.message_offset + messages.len() < c.messages.len()).then_some(query.message_offset + messages.len());
            json::json!({
                "thread":{"id":c.id,"anchor":c.anchor,"layer":c.layer,"resolved":c.resolved,"messages":messages},
                "message_count":c.messages.len(),"next_message_offset":next_message,
                "anchor_state":match s.anchor_state(c) { AnchorState::Visible=>"visible", AnchorState::Orphaned=>"orphaned", AnchorState::Hidden=>"hidden" }
            })
        }).collect();
        let next =
            (query.offset + page.len() < comments.len()).then_some(query.offset + page.len());
        json::json!({"comments":page,"total":comments.len(),"next_offset":next})
    }
}

impl Tool for CommentsPlugin {
    fn interact(&self, cx: &dyn ToolCx, input: ToolInput) -> Result<ToolOutcome, ToolError> {
        let target = match input.gesture {
            ToolGesture::ApplyAt(at) => Some(at),
            ToolGesture::Preview(at) => at,
            ToolGesture::Hover | ToolGesture::Click => cx.hovered_hit().map(|hit| hit.at),
        };
        Ok(ToolOutcome {
            target,
            ..Default::default()
        })
    }
}
