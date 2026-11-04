//! Software updates, background check at startup, and startup prompt modal.

use crate::WordApp;

#[cfg(all(not(target_arch = "wasm32"), not(target_env = "ohos")))]
use craft_update::dialog::{CheckResultDialog, CheckResultResponse};
#[cfg(all(not(target_arch = "wasm32"), not(target_env = "ohos")))]
use craft_update::updater::{Action, Status, Updater};

pub(crate) struct Runtime {
    #[cfg(all(not(target_arch = "wasm32"), not(target_env = "ohos")))]
    updater: Updater,
    #[cfg(all(not(target_arch = "wasm32"), not(target_env = "ohos")))]
    result_dialog: CheckResultDialog,
    /// The desktop app turns the check at start on; tests and the web build leave it off.
    #[cfg(all(not(target_arch = "wasm32"), not(target_env = "ohos")))]
    startup_check: bool,
    #[cfg(all(not(target_arch = "wasm32"), not(target_env = "ohos")))]
    check_requested: bool,
    /// The context of the last frame, to wake the UI when a check is queued from a menu callback.
    #[cfg(all(not(target_arch = "wasm32"), not(target_env = "ohos")))]
    repaint: Option<egui::Context>,
    pub(crate) manual_window_open: bool,
    #[cfg(any(target_arch = "wasm32", target_env = "ohos"))]
    pub(crate) status_message: Option<String>,
}

#[cfg_attr(any(target_arch = "wasm32", target_env = "ohos"), allow(clippy::derivable_impls))]
impl Default for Runtime {
    fn default() -> Self {
        Self {
            #[cfg(all(not(target_arch = "wasm32"), not(target_env = "ohos")))]
            updater: Updater::new(&crate::UPDATE_APP, env!("CARGO_PKG_VERSION")),
            #[cfg(all(not(target_arch = "wasm32"), not(target_env = "ohos")))]
            result_dialog: CheckResultDialog::new(&crate::UPDATE_APP, env!("CARGO_PKG_VERSION")),
            #[cfg(all(not(target_arch = "wasm32"), not(target_env = "ohos")))]
            startup_check: false,
            #[cfg(all(not(target_arch = "wasm32"), not(target_env = "ohos")))]
            check_requested: false,
            #[cfg(all(not(target_arch = "wasm32"), not(target_env = "ohos")))]
            repaint: None,
            manual_window_open: false,
            #[cfg(any(target_arch = "wasm32", target_env = "ohos"))]
            status_message: None,
        }
    }
}

impl WordApp {
    /// Whether the app asks for a newer release when it starts (off until the desktop app turns it on).
    #[cfg(all(not(target_arch = "wasm32"), not(target_env = "ohos")))]
    pub fn set_startup_update_check(&mut self, on: bool) {
        self.updates_rt.startup_check = on;
    }
}

pub(crate) fn check(app: &mut WordApp) {
    #[cfg(all(not(target_arch = "wasm32"), not(target_env = "ohos")))]
    {
        app.updates_rt.check_requested = true;
        app.updates_rt.manual_window_open = true;
        if let Some(ctx) = &app.updates_rt.repaint {
            ctx.request_repaint();
        }
    }
    #[cfg(any(target_arch = "wasm32", target_env = "ohos"))]
    {
        app.updates_rt.manual_window_open = true;
        #[cfg(target_arch = "wasm32")]
        let message = tl!("Update check is not supported in the web browser.");
        #[cfg(target_env = "ohos")]
        let message = "Updates are managed by the host application.";
        app.updates_rt.status_message = Some(message.to_string());
    }
}

pub(crate) fn tick(app: &mut WordApp, ctx: &egui::Context) {
    #[cfg(all(not(target_arch = "wasm32"), not(target_env = "ohos")))]
    {
        let rt = &mut app.updates_rt;
        rt.repaint = Some(ctx.clone());
        if std::mem::take(&mut rt.check_requested) {
            rt.updater.check_now(ctx);
        } else if rt.startup_check {
            rt.updater.check_at_start(ctx);
        }
        rt.updater.tick(ctx);
    }
    #[cfg(any(target_arch = "wasm32", target_env = "ohos"))]
    let _ = (app, ctx);
}

pub(crate) fn show(app: &mut WordApp, ctx: &egui::Context) {
    #[cfg(all(not(target_arch = "wasm32"), not(target_env = "ohos")))]
    {
        // The restart helper is scheduled before the app closes, so no update starts over unsaved work.
        let clean = !(app.session.dirty || app.save_unconfirmed() || app.printing.busy() || crate::panes::has_comment_drafts(app));
        match app.updates_rt.updater.show_if(ctx, clean) {
            Action::OpenUrl(url) => ctx.open_url(egui::OpenUrl::new_tab(url)),
            Action::Close => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            Action::None => {}
        }
        // The dialog answers Help ▸ Check for Updates; a found update leaves the status idle and the prompt above takes over.
        let rt = &mut app.updates_rt;
        if rt.manual_window_open {
            match rt.result_dialog.show(ctx, rt.updater.status()) {
                CheckResultResponse::None => {}
                CheckResultResponse::Close => {
                    rt.manual_window_open = false;
                    rt.updater.dismiss_status();
                }
                CheckResultResponse::CheckAgain => rt.updater.check_now(ctx),
            }
            // An update was found: the prompt replaces the dialog.
            if matches!(rt.updater.status(), Status::Idle) {
                rt.manual_window_open = false;
            }
        }
    }

    #[cfg(any(target_arch = "wasm32", target_env = "ohos"))]
    if app.updates_rt.manual_window_open {
        let mut open = true;
        egui::Window::new(tl!("Check for Updates"))
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                crate::rtl::root(ui, |ui| {
                    if let Some(msg) = &app.updates_rt.status_message {
                        ui.label(msg);
                    }
                })
            });
        if !open {
            app.updates_rt.manual_window_open = false;
            app.updates_rt.status_message = None;
        }
    }
}
