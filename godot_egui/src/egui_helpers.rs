pub fn progress_bar(ui: &mut egui::Ui, progress: f32, font_size: f32) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(8.0, 0.8);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

    let visuals = ui.style().interact(&response);
    let rect = rect.expand(visuals.expansion);
    let radius = 0.15 * rect.height();
    ui.painter().rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);

    if progress > 0.0 {
        let progress_rect =
            egui::Rect { min: rect.min, max: egui::Pos2::new(rect.min.x + rect.width() * progress, rect.max.y) };
        ui.painter().rect(progress_rect, radius, visuals.fg_stroke.color, visuals.fg_stroke);
    }

    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        format!("{:.2}", progress),
        egui::FontId::proportional(font_size),
        ui.style().visuals.text_color(),
    );

    response
}

pub const NUMBER_KEYS: [egui::Key; 10] = [
    egui::Key::Num1,
    egui::Key::Num2,
    egui::Key::Num3,
    egui::Key::Num4,
    egui::Key::Num5,
    egui::Key::Num6,
    egui::Key::Num7,
    egui::Key::Num8,
    egui::Key::Num9,
    egui::Key::Num0,
];

pub trait ColorHelpers<T> {
    fn with_alpha(&self, alpha: T) -> Self;
    fn lightened(&self, amount: f32) -> Self;
}
impl ColorHelpers<u8> for egui::Color32 {
    fn with_alpha(&self, alpha: u8) -> Self {
        let mut color = *self;
        color[3] = alpha;
        color
    }
    fn lightened(&self, amount: f32) -> Self {
        Self::from_rgba_premultiplied(
            (self.r() as f32 * amount) as u8,
            (self.g() as f32 * amount) as u8,
            (self.b() as f32 * amount) as u8,
            self.a(),
        )
    }
}
impl ColorHelpers<u8> for egui::Stroke {
    fn with_alpha(&self, alpha: u8) -> Self {
        let mut color = self.color;
        color[3] = alpha;
        Self { width: self.width, color }
    }
    fn lightened(&self, amount: f32) -> Self {
        Self { width: self.width, color: self.color.lightened(amount) }
    }
}
