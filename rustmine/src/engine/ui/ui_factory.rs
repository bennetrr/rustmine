use egui::{
    Align, Align2, Area, Button, Color32, FontData, FontDefinitions, FontFamily, Id, Response,
    RichText, Stroke, Ui, Vec2, Visuals, vec2,
};
use std::string::ToString;

pub static COLOR_INACTIVE: Color32 = Color32::from_rgb(85, 65, 55);
pub static COLOR_ACTIVE: Color32 = Color32::from_rgb(146, 155, 206);
pub static COLOR_HOVERED: Color32 = Color32::from_rgb(146, 155, 206);
pub static COLOR_BORDER_LIGHT: Color32 = Color32::from_rgb(255, 255, 255);
pub static COLOR_BORDER_DARK: Color32 = Color32::from_rgb(34, 34, 34);
pub static COLOR_TEXT: Color32 = Color32::WHITE;

pub static FONT_BOLD: &str = "bold";
pub static FONT_BOLD_ITALIC: &str = "bold-italic";
pub static FONT_ITALIC: &str = "italic";
pub static FONT_REGULAR: &str = "regular";
pub static FONT_TITLE: &str = "title";

/// Viewport-aware sizing and egui widget helpers.
pub struct UiComponents {
    width_pt: f32,
    height_pt: f32,

    pub button_height: f32,
    pub button_font_size: f32,
    pub heading_font_size: f32,
}

impl UiComponents {
    /// # Arguments
    /// - `px_per_pt` — the display scale factor (physical pixels per egui point)
    pub fn new(width_px: u32, height_px: u32, px_per_pt: f32) -> Self {
        UiComponents {
            width_pt: width_px as f32 / px_per_pt,
            height_pt: height_px as f32 / px_per_pt,

            button_height: 80.0,
            button_font_size: 45.0,
            heading_font_size: 70.0,
        }
    }

    /// Get a size in points (pt)
    pub fn pt(&self, pt: u32) -> f32 {
        pt as f32
    }

    /// Get a size in percent of the viewport width
    pub fn vw(&self, percent: u32) -> f32 {
        self.width_pt * (percent as f32 / 100.0)
    }

    /// Get a size in percent of the viewport height
    pub fn vh(&self, percent: u32) -> f32 {
        self.height_pt * (percent as f32 / 100.0)
    }

    /// Set a horizontal gap between items of the container
    pub fn gap_x(&self, ui: &mut Ui, size: f32) {
        ui.spacing_mut().item_spacing.x = size;
    }

    /// Set a vertical gap between items of the container
    pub fn gap_y(&self, ui: &mut Ui, size: f32) {
        ui.spacing_mut().item_spacing.y = size;
    }

    /// Add an aligned area without an offset to the container
    pub fn area(&self, ui: &mut Ui, id: &str, align: Align2, show: impl FnOnce(&mut Ui)) {
        self.area_offset(ui, id, align, vec2(0.0, 0.0), show);
    }

    /// Add an aligned area with an offset to the container
    pub fn area_offset(
        &self,
        ui: &mut Ui,
        id: &str,
        align: Align2,
        offset: Vec2,
        show: impl FnOnce(&mut Ui),
    ) {
        Area::new(Id::new(id)).anchor(align, offset).show(ui, show);
    }

    /// Add a heading with the specified text to the container
    pub fn heading(&self, ui: &mut Ui, text: &str) {
        self.area_offset(
            ui,
            format!("heading-{}", text).as_str(),
            Align2::CENTER_TOP,
            vec2(self.pt(0), self.pt(40)),
            |ui| {
                let visuals = self.visuals(ui);
                ui.set_visuals(visuals);

                ui.scope(|ui| {
                    ui.horizontal_centered(|ui| {
                        ui.heading(
                            RichText::new(text)
                                .color(COLOR_TEXT)
                                .size(self.heading_font_size),
                        )
                    })
                });
            },
        );
    }

    /// Add a label with the specified text to the container
    pub fn label(&self, ui: &mut Ui, text: &str, size: f32) {
        ui.label(RichText::new(text).color(COLOR_TEXT).size(size));
    }

    /// Add a button to the container
    pub fn button(&self, ui: &mut Ui, width: f32, label: &str) -> Response {
        ui.add_sized(
            vec2(width, self.button_height),
            Button::new(
                RichText::new(label)
                    .color(COLOR_TEXT)
                    .strong()
                    .size(self.button_font_size),
            ),
        )
    }

    /// Clone the visuals and apply the visuals for a widgets group
    pub fn visuals(&self, ui: &Ui) -> Visuals {
        let mut visuals = ui.visuals().clone();
        visuals.panel_fill = Color32::TRANSPARENT;

        visuals.widgets.inactive.weak_bg_fill = COLOR_INACTIVE;
        visuals.widgets.inactive.bg_fill = COLOR_INACTIVE;
        visuals.widgets.inactive.bg_stroke = Stroke::new(self.pt(2), COLOR_BORDER_DARK);
        visuals.widgets.inactive.corner_radius = 0.0.into();

        visuals.widgets.hovered.weak_bg_fill = COLOR_HOVERED;
        visuals.widgets.hovered.bg_fill = COLOR_HOVERED;
        visuals.widgets.hovered.bg_stroke = Stroke::new(self.pt(2), COLOR_BORDER_LIGHT);
        visuals.widgets.hovered.corner_radius = 0.0.into();

        visuals.widgets.active.weak_bg_fill = COLOR_ACTIVE;
        visuals.widgets.active.bg_fill = COLOR_ACTIVE;
        visuals.widgets.active.bg_stroke = Stroke::new(self.pt(2), COLOR_BORDER_LIGHT);
        visuals.widgets.active.fg_stroke = Stroke::new(self.pt(1), Color32::WHITE);
        visuals.widgets.active.corner_radius = 0.0.into();

        // inputs
        visuals.text_cursor.stroke.width = self.pt(2);
        visuals.text_cursor.stroke.color = COLOR_ACTIVE;
        visuals.text_cursor.blink = true;

        visuals
    }

    /// Renders a single-line text input sized to `width × height`.
    ///
    /// # Arguments
    /// - `char_limit` — maximum character count; `None` means unlimited
    /// - `return_variable` — the `String` backing the input field; mutated in place
    pub fn input(
        &self,
        ui: &mut Ui,
        width: f32,
        height: f32,
        char_limit: Option<usize>,
        return_variable: &mut String,
    ) -> Response {
        ui.add_sized(
            vec2(width, height),
            egui::TextEdit::singleline(return_variable)
                .font(egui::FontId::proportional(height))
                .margin(vec2(4.0, 10.0))
                .cursor_at_end(true)
                .vertical_align(Align::Center)
                .char_limit(char_limit.unwrap_or(usize::MAX))
                .text_color(COLOR_TEXT)
                .interactive(true),
        )
    }
}

macro_rules! include_font {
    ($path:literal) => {
        FontData::from_static(include_bytes!($path)).into()
    };
}

pub fn load_fonts() -> Option<FontDefinitions> {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        FONT_BOLD.to_string(),
        include_font!("../../assets/fonts/bold.ttf"),
    );
    log::debug!("Loaded font {FONT_BOLD}");
    fonts.font_data.insert(
        FONT_BOLD_ITALIC.to_string(),
        include_font!("../../assets/fonts/bold-italic.ttf"),
    );
    log::debug!("Loaded font {FONT_BOLD_ITALIC}");
    fonts.font_data.insert(
        FONT_ITALIC.to_string(),
        include_font!("../../assets/fonts/italic.ttf"),
    );
    log::debug!("Loaded font {FONT_ITALIC}");
    fonts.font_data.insert(
        FONT_REGULAR.to_string(),
        include_font!("../../assets/fonts/regular.ttf"),
    );
    log::debug!("Loaded font {FONT_REGULAR}");
    fonts.font_data.insert(
        FONT_TITLE.to_string(),
        include_font!("../../assets/fonts/title.ttf"),
    );
    log::debug!("Loaded font {FONT_TITLE}");

    fonts
        .families
        .get_mut(&FontFamily::Proportional)?
        .insert(0, FONT_REGULAR.to_string());
    log::debug!("Registered font {FONT_REGULAR}");
    fonts
        .families
        .get_mut(&FontFamily::Proportional)?
        .insert(1, FONT_BOLD.to_string());
    log::debug!("Registered font {FONT_BOLD}");
    fonts
        .families
        .get_mut(&FontFamily::Proportional)?
        .insert(2, FONT_ITALIC.to_string());
    log::debug!("Registered font {FONT_ITALIC}");
    fonts
        .families
        .get_mut(&FontFamily::Proportional)?
        .insert(3, FONT_BOLD_ITALIC.to_string());
    log::debug!("Registered font {FONT_BOLD_ITALIC}");
    fonts.families.insert(
        FontFamily::Name(FONT_TITLE.into()),
        vec![FONT_TITLE.to_string()],
    );
    log::debug!("Registered font {FONT_TITLE}");

    Some(fonts)
}
