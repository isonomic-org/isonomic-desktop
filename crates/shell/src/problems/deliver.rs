//! Sending reports, once the build has a destination: Send on the notice,
//! or every kept report under Send automatically.
//!
//! One attempt, then, on a network error or a 5xx, more with a doubling
//! wait for up to a day ([`reports::send::next_wait`]); a 4xx is never
//! tried again. A 2xx deletes the kept file. Whatever happens, a report
//! that isn't sent stays kept, and is asked about (or sent) at the next
//! look. After Send, the person sees "Sent" or "Couldn't send; kept on
//! this Mac" from the first attempt; later retries are quiet. The window
//! log gets the status, never the body.

use std::sync::Arc;
use std::time::Instant;

use reports::send::{next_wait, Attempt, Sender};
use reports::store::Pending;
use ui::component::notification::Notification;
use ui::component::WindowExt as _;
use ui::{AppContext as _, Context, Window};

use super::ProblemReports;

/// What the person sees after Send.
pub const SENT: &str = "Sent";
pub const NOT_SENT: &str = "Couldn't send; kept on this Mac";

impl ProblemReports {
    /// Send reports with `sender`, to `host` (shown on the setting).
    pub fn send_with(&mut self, sender: Arc<dyn Sender>, host: String, cx: &mut Context<Self>) {
        self.sender = Some((sender, host));
        cx.notify();
    }

    /// How the last report the person sent went, from its first attempt.
    pub fn last_outcome(&self) -> Option<&'static str> {
        self.last_outcome
    }

    /// Send: the report on screen goes, exactly as Show report showed it.
    pub fn send_current(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.sender.is_none() {
            return;
        }
        let Some(body) = self.outgoing() else {
            return;
        };
        let Some(report) = self.current.take() else {
            return;
        };
        let id = std::mem::take(&mut self.current_id);
        self.asked.insert(report.path.clone());
        self.deliver(report, body, id, true, window, cx);
        cx.notify();
        self.review(window, cx);
    }

    /// Post `body` for `report`, retrying as above. `tell` shows the first
    /// attempt's outcome.
    pub(super) fn deliver(
        &mut self,
        report: Pending,
        body: String,
        id: String,
        tell: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some((sender, _)) = self.sender.clone() else {
            return;
        };
        if !self.sending.insert(report.path.clone()) {
            return; // already on its way
        }
        let kept = Arc::clone(&self.kept);
        cx.spawn_in(window, async move |this, cx| {
            let started = Instant::now();
            let mut tries = 0;
            let outcome = loop {
                tries += 1;
                let attempt = sender.send(body.clone(), id.clone()).await;
                if tell && tries == 1 {
                    let shown = if attempt == Attempt::Sent {
                        SENT
                    } else {
                        NOT_SENT
                    };
                    let _ = this.update_in(cx, |this, window, cx| {
                        this.last_outcome = Some(shown);
                        let note = if attempt == Attempt::Sent {
                            Notification::success(shown)
                        } else {
                            Notification::warning(shown)
                        };
                        window.push_notification(note, cx);
                        cx.notify();
                    });
                }
                match attempt {
                    Attempt::Failed(why) => match next_wait(tries, started.elapsed()) {
                        Some(wait) => {
                            reports::log::line(&format!(
                                "report not sent yet ({why}); trying again"
                            ));
                            cx.background_executor().timer(wait).await;
                        }
                        None => break Attempt::Failed(why),
                    },
                    done => break done,
                }
            };
            match &outcome {
                Attempt::Sent => {
                    reports::log::line("report sent");
                    let gone = report.clone();
                    let _ = cx
                        .background_spawn(async move { kept.discard(&gone) })
                        .await;
                }
                Attempt::Refused(code) => {
                    reports::log::line(&format!("report refused with status {code}; kept"));
                }
                Attempt::Failed(why) => {
                    reports::log::line(&format!("report not sent after a day ({why}); kept"));
                }
            }
            let _ = this.update(cx, |this, cx| {
                this.sending.remove(&report.path);
                cx.notify();
            });
        })
        .detach();
    }
}
