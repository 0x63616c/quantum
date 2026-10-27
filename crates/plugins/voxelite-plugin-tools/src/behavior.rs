use voxelite_plugin_api::{
    Command, Mirror, Mode, PaletteEdit, Pos, Region, Shape, Tool, ToolCx, ToolError, ToolGesture,
    ToolInput, ToolOutcome, Voxel,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BuiltinTool {
    Place,
    Erase,
    Paint,
    Fill,
    Box,
    Picker,
}

impl BuiltinTool {
    pub(crate) const ALL: [Self; 6] = [
        Self::Place,
        Self::Erase,
        Self::Paint,
        Self::Fill,
        Self::Box,
        Self::Picker,
    ];

    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::Place => "dev.voxelite.tools.place.tool",
            Self::Erase => "dev.voxelite.tools.erase.tool",
            Self::Paint => "dev.voxelite.tools.paint.tool",
            Self::Fill => "dev.voxelite.tools.fill.tool",
            Self::Box => "dev.voxelite.tools.box.tool",
            Self::Picker => "dev.voxelite.tools.picker.tool",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Place => "Place",
            Self::Erase => "Erase",
            Self::Paint => "Paint",
            Self::Fill => "Fill",
            Self::Box => "Box",
            Self::Picker => "Picker",
        }
    }

    fn target(self, cx: &dyn ToolCx) -> Option<Pos> {
        let adds_cells = matches!(self, Self::Place | Self::Box);
        if let Some(hit) = cx.hovered_hit() {
            let at = if adds_cells {
                std::array::from_fn(|a| hit.at[a] + hit.normal[a])
            } else {
                hit.at
            };
            cx.bounds().contains(at).then_some(at)
        } else if adds_cells {
            cx.ground_target()
        } else {
            None
        }
    }

    fn apply(
        self,
        cx: &dyn ToolCx,
        at: Pos,
        mut outcome: ToolOutcome,
    ) -> Result<ToolOutcome, ToolError> {
        if !cx.bounds().contains(at) {
            return Err(ToolError::Gesture("Outside document bounds".into()));
        }
        if self == Self::Picker {
            outcome.sampled_color = cx
                .sample(at)
                .map(|(slot, _)| cx.palette().get(slot).expect("valid slot").clone());
            return Ok(outcome);
        }
        let layer = cx.layer(cx.active_layer())?;
        if layer.locked || !layer.visible {
            return Err(ToolError::Gesture(
                "The active layer is locked or hidden".into(),
            ));
        }
        if self == Self::Box && outcome.anchor.is_none() {
            outcome.anchor = Some(at);
            return Ok(outcome);
        }
        // A no-op brush must not add unused palette entries.
        if matches!(self, Self::Paint | Self::Erase | Self::Fill)
            && cx.voxel(cx.active_layer(), at)? == 0
        {
            return Ok(outcome);
        }
        let slot = if self == Self::Erase {
            0
        } else if let Some(s) = cx
            .palette()
            .slots
            .iter()
            .find(|s| s.color.rgba == cx.active_color().rgba)
        {
            s.index
        } else {
            let slot = (1..=256)
                .find(|s| cx.palette().get(*s).is_none())
                .ok_or_else(|| ToolError::Gesture("Palette is full (256 colors)".into()))?;
            outcome.commands.push(Command::Palette(PaletteEdit::Add {
                slot: Some(slot),
                color: cx.active_color().clone(),
            }));
            slot
        };
        // Mirror settings belong to the host; tools always propose an unmirrored edit.
        let mirror = Mirror::default();
        outcome.commands.push(match self {
            Self::Fill => Command::FloodFill {
                layer: cx.active_layer(),
                seed: at,
                slot,
                region: None,
                cap: 1_048_576,
                mirror,
            },
            Self::Box => Command::Primitive {
                layer: cx.active_layer(),
                shape: Shape::Box {
                    region: preview(Some(at), outcome.anchor).expect("box start established"),
                    hollow: false,
                },
                mode: Mode::Place,
                slot: Some(slot),
                mirror,
            },
            _ => Command::SetVoxels {
                layer: cx.active_layer(),
                voxels: vec![Voxel { at, slot }],
                mode: match self {
                    Self::Erase => Mode::Erase,
                    Self::Paint => Mode::Paint,
                    _ => Mode::Place,
                },
                mirror,
            },
        });
        outcome.anchor = None;
        outcome.preview = preview(outcome.target, outcome.anchor);
        Ok(outcome)
    }
}

fn preview(hover: Option<Pos>, anchor: Option<Pos>) -> Option<Region> {
    let at = hover.or(anchor)?;
    Some(Region {
        min: anchor.map_or(at, |s| std::array::from_fn(|a| s[a].min(at[a]))),
        max: anchor.map_or(at.map(|v| v + 1), |s| {
            std::array::from_fn(|a| s[a].max(at[a]) + 1)
        }),
    })
}

impl Tool for BuiltinTool {
    fn interact(&self, cx: &dyn ToolCx, input: ToolInput) -> Result<ToolOutcome, ToolError> {
        // Validate keyboard coordinates before calculating the half-open preview.
        if let ToolGesture::ApplyAt(at) = input.gesture
            && !cx.bounds().contains(at)
        {
            return Err(ToolError::Gesture("Outside document bounds".into()));
        }
        let target = match input.gesture {
            ToolGesture::Preview(at) => at,
            ToolGesture::ApplyAt(at) => Some(at),
            _ => self.target(cx),
        };
        let mut outcome = ToolOutcome {
            label: self.label().into(),
            target,
            anchor: input.anchor,
            preview: preview(target, input.anchor),
            ..Default::default()
        };
        match input.gesture {
            ToolGesture::Hover | ToolGesture::Preview(_) => Ok(outcome),
            ToolGesture::Click if *self == Self::Picker => {
                outcome.sampled_color = cx
                    .hovered_hit()
                    .map(|hit| cx.palette().get(hit.slot).expect("valid slot").clone());
                Ok(outcome)
            }
            ToolGesture::Click => {
                if let Some(hit) = cx.hovered_hit()
                    && hit.layer != cx.active_layer()
                    && matches!(self, Self::Paint | Self::Erase | Self::Fill)
                {
                    return Err(ToolError::Gesture(
                        "Select the hit voxel’s layer to edit it".into(),
                    ));
                }
                match target {
                    Some(at) => self.apply(cx, at, outcome),
                    None => Ok(outcome),
                }
            }
            ToolGesture::ApplyAt(at) => self.apply(cx, at, outcome),
        }
    }
}

#[cfg(test)]
mod tests;
