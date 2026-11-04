//! An embedding Ability owns one surface. Documents use the existing single-window
//! switching path in `documents`, without the desktop viewport manager.

use crate::{WordApp, window_ui::WindowUi};
use serde_json::Value;

pub struct Windows;

impl Windows {
    pub(crate) fn new(_doc: u64) -> Self {
        Self
    }

    pub(crate) fn native(&self) -> bool {
        false
    }

    pub(crate) fn driven(&self) -> bool {
        false
    }

    pub(crate) fn shown(&self) -> u64 {
        1
    }
}

impl WordApp {
    /// The host owns a single native surface.
    pub fn window_count(&self) -> usize {
        1
    }

    /// All UI and control input belong to the host's root viewport.
    pub fn active_viewport(&self) -> egui::ViewportId {
        egui::ViewportId::ROOT
    }

    pub(crate) fn active_surface_geometry(&self, _ctx: &egui::Context) -> (egui::ViewportId, Option<crate::control::SurfaceGeometry>) {
        (self.active_viewport(), self.surface_geometry)
    }

    /// Additional native windows are unavailable on the host's single surface.
    pub fn set_window_builder(&mut self, _builder: egui::ViewportBuilder) {}

    /// Closing the root surface closes the application, after its save prompts.
    pub fn closes_a_window(&self) -> bool {
        false
    }

    /// Ask about unsaved documents before allowing the host to quit.
    pub fn quit(&mut self) -> bool {
        let go = self.close_requested();
        if go {
            self.quit_requested = true;
        }
        go
    }

    pub(crate) fn new_window(&mut self) {}

    pub(crate) fn closes_only_its_window(&self, _id: &str, _params: &Value) -> bool {
        false
    }

    pub(crate) fn close_window_only(&mut self) -> bool {
        false
    }

    pub(crate) fn prune_windows(&mut self) {}

    pub(crate) fn follow_in_windows(&mut self, _id: u64) -> bool {
        false
    }

    pub(crate) fn window_shows(&mut self, _id: u64) {}

    pub(crate) fn close_shown_document(&mut self) -> Result<(), String> {
        self.session.close_document(self.session.document_id()).map_err(|e| e.to_string())?;
        self.sync_document();
        Ok(())
    }

    pub(crate) fn queue_for_window(&mut self, _document: u64, _req: &Value) -> bool {
        false
    }

    pub(crate) fn switch_windows_entries(&self) -> Vec<(u64, String, bool)> {
        Vec::new()
    }

    pub(crate) fn show_window(&mut self, _key: u64) {}

    pub(crate) fn window_title(&self) -> String {
        if !self.has_document() {
            return self.display_name().replace('\0', "");
        }
        format!("{}{} - {}", self.title_stem(), if self.session.dirty { " •" } else { "" }, self.display_name()).replace('\0', "")
    }

    pub(crate) fn windows_ui(&mut self, ui: &mut egui::Ui) {
        self.window_ui(ui);
    }

    pub(crate) fn send_control_input(&mut self, _ctx: &egui::Context) {}

    pub(crate) fn control_input_is_roots(&self, _ctx: &egui::Context) -> bool {
        true
    }

    pub(crate) fn window_id(&self, name: &'static str) -> egui::Id {
        egui::Id::unique(name).with(self.windows.shown())
    }

    pub(crate) fn other_windows(&self) -> impl Iterator<Item = &WindowUi> {
        std::iter::empty()
    }
}
