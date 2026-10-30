//! Discrete camera navigation; direct gestures continue to update the rig immediately.
use super::*;
use voxelite_app::reduce_motion::{camera_animation_enabled, prefers_reduced_motion};
use voxelite_render::DEFAULT_TRANSITION;

impl EditorView {
    pub(super) fn animate_camera_to(&mut self, destination: OrbitCamera, cx: &mut Context<Self>) {
        if !camera_animation_enabled(self.preferences.animate_camera, prefers_reduced_motion())
            || !self.viewport.has_frame()
            || !self.camera_anim.start(
                self.editor.camera.pose(),
                destination.pose(),
                self.anim_epoch.elapsed(),
                DEFAULT_TRANSITION,
            )
        {
            self.cancel_camera_animation();
            self.viewport.end_navigation();
            self.editor.camera = destination;
            self.request_render(cx);
        } else {
            self.sample_camera_animation(cx);
            // The command has no Window; notify enters layout, which schedules
            // the next animation frame using that window's frame loop.
            cx.notify();
        }
    }

    pub(super) fn tick_camera_animation(&mut self, window: &Window, cx: &mut Context<Self>) {
        if !self.camera_anim.is_animating() {
            return;
        }
        if self.sample_camera_animation(cx) == Some(false) {
            window.request_animation_frame();
        }
    }

    fn sample_camera_animation(&mut self, cx: &mut Context<Self>) -> Option<bool> {
        let sample = self.camera_anim.sample(self.anim_epoch.elapsed())?;
        self.editor.camera = OrbitCamera::from_pose(sample.pose);
        self.request_navigation(cx);
        if sample.finished {
            // Settle the CPU image now and cancel its delayed refinement. GPU
            // navigation already requested the exact final pose above.
            self.viewport.end_navigation();
            if !self.viewport.gpu_enabled() {
                self.request_render(cx);
            }
        }
        Some(sample.finished)
    }

    pub(super) fn cancel_camera_animation(&mut self) {
        if self.camera_anim.is_animating() {
            self.camera_anim.cancel();
            self.viewport.end_navigation();
        }
    }
}
