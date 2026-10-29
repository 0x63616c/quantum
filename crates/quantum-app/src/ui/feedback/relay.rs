use super::*;

impl FeedbackView {
    pub(super) fn privacy(&self) -> String {
        match &self.endpoint {
            None => PRIVACY.into(),
            Some(endpoint) => format!(
                "Nothing is sent automatically. Send public report sends this preview to {endpoint} (Cloudflare), then a public issue in github.com/0x63616c/quantum. Includes version, OS, architecture and the displayed diagnostics/context. The relay stores no reports or contact email; GitHub retains public issues until removed. Hourly IP hashes expire at hour boundaries, subject to cleanup delivery; Cloudflare processes network metadata under its own policy. Email and scene attachments are unavailable. Local crash reports stay until deleted. Review for private details."
            ),
        }
    }

    pub(super) fn submit(&mut self, cx: &mut Context<Self>) {
        if self.sending || self.preview.is_none() {
            return;
        }
        let Some(endpoint) = &self.endpoint else {
            return;
        };
        let request = diagnostics::redactor()
            .ok()
            .and_then(|redactor| {
                self.report
                    .payload(
                        if self.is_crash() {
                            "fix: Quantum crash report"
                        } else {
                            "Quantum feedback"
                        },
                        &redactor,
                    )
                    .ok()
            })
            .and_then(|payload| payload.request(endpoint, env!("CARGO_PKG_VERSION")).ok());
        let Some(request) = request else {
            self.error = Some("Invalid relay configuration or report. Nothing was sent. Copy or save your preview.".into());
            cx.notify();
            return;
        };
        self.sending = true;
        self.error = None;
        let task = cx
            .background_executor()
            .spawn(async move { request.send(&diagnostics::Transport).is_ok() });
        cx.spawn(async move |this, cx| {
            let accepted = task.await;
            let _ = this.update(cx, |this, cx| {
                this.sending = false;
                this.error = Some(if accepted {
                    "GitHub confirmed issue creation. Local crash reports are kept until you choose Delete report."
                } else {
                    "Submission could not be confirmed. Your report is kept. Nothing will retry automatically. GitHub may have created an issue if its reply was lost; check before sending again. Copy or save the preview."
                }.into());
                cx.notify();
            });
        }).detach();
        cx.notify();
    }
}
