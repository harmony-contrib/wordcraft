//! Optional identity for applications embedding the editor.

/// Host-owned identity; contributor and model credits still describe WordCraft.
#[derive(Clone, Copy, Debug)]
pub struct Branding {
    pub name: &'static str,
    pub icon_png: &'static [u8],
    pub attribution: &'static str,
}

impl crate::WordApp {
    /// The embedding application's name, or the upstream name when no brand is supplied.
    pub fn display_name(&self) -> &'static str {
        self.services.branding.map_or("WordCraft", |brand| brand.name)
    }

    pub(crate) fn brand_text<'a>(&self, text: &'a str) -> std::borrow::Cow<'a, str> {
        match self.services.branding {
            Some(brand) => text.replace("WordCraft", brand.name).into(),
            None => text.into(),
        }
    }
}

impl Branding {
    fn texture(&self, ctx: &egui::Context) -> egui::TextureHandle {
        let id = egui::Id::unique(("wordcraft-host-icon", self.icon_png.as_ptr(), self.icon_png.len()));
        if let Some(texture) = ctx.data(|data| data.get_temp::<egui::TextureHandle>(id)) {
            return texture;
        }
        let mut reader = image::ImageReader::with_format(std::io::Cursor::new(self.icon_png), image::ImageFormat::Png);
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(1024);
        limits.max_image_height = Some(1024);
        limits.max_alloc = Some(16 * 1024 * 1024);
        reader.limits(limits);
        let image = reader.decode().map_or_else(
            |_| egui::ColorImage::filled([1, 1], egui::Color32::TRANSPARENT),
            |image| {
                let image = image.to_rgba8();
                egui::ColorImage::from_rgba_unmultiplied([image.width() as usize, image.height() as usize], image.as_raw())
            },
        );
        let texture = ctx.load_texture("wordcraft-host-icon", image, egui::TextureOptions::LINEAR);
        ctx.data_mut(|data| data.insert_temp(id, texture.clone()));
        texture
    }

    pub(crate) fn show_icon(&self, ui: &mut egui::Ui, size: f32) {
        let texture = self.texture(ui.ctx());
        ui.image((texture.id(), egui::vec2(size, size)));
    }
}

#[cfg(test)]
mod tests {
    use super::Branding;
    use crate::{Services, WordApp};

    #[test]
    fn host_identity_keeps_document_names_and_handles_invalid_icons() {
        let brand = Branding { name: "Quilrune", icon_png: b"invalid PNG", attribution: "Based on WordCraft" };
        let mut app = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        assert_eq!(app.display_name(), "WordCraft");
        app.services.branding = Some(brand);
        app.session.path = Some("WordCraft notes.docx".into());
        assert_eq!(app.display_name(), "Quilrune");
        assert_eq!(app.window_title(), "WordCraft notes - Quilrune");
        assert_eq!(app.brand_text("About WordCraft"), "About Quilrune");
        let context = egui::Context::default();
        assert_eq!(brand.texture(&context).size(), [1, 1]);
        assert_eq!(brand.texture(&context).id(), brand.texture(&context).id());
        let other = Branding { icon_png: b"another icon", ..brand };
        assert_ne!(brand.texture(&context).id(), other.texture(&context).id());
    }

    #[test]
    fn about_shows_host_attribution_and_upstream_credits() {
        use egui_kittest::kittest::Queryable;
        let mut app = WordApp::new(
            wordcraft_engine::Session::new(wordcraft_doc::Document::new()),
            Services { branding: Some(Branding { name: "Quilrune", icon_png: &[], attribution: "Based on WordCraft" }), ..Default::default() },
        );
        app.dialog = Some(crate::dialogs::Dialog::About { tab: 0 });
        let mut harness = egui_kittest::Harness::builder().with_size(egui::vec2(1100.0, 800.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
        harness.run();
        assert!(harness.query_by_label("Quilrune").is_some());
        assert!(harness.query_by_label("Based on WordCraft").is_some());
        assert!(harness.query_by_label("Contributors").is_some());
        assert!(harness.query_by_label("MIT OR Apache-2.0. Copyright (c) 2026 ArtCraft Team and the WordCraft contributors.").is_some());
    }
}
