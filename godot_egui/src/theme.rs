use egui_stylist::EguiTheme;
use gdnative::api::Resource;
use gdnative::prelude::*;

/// Holds the serialized data of an `egui_stylist::EguiTheme`.
/// This allows Godot
#[derive(NativeClass)]
#[inherit(Resource)]
pub struct GodotEguiTheme {
    #[property]
    pub serialized_theme: String,
}

#[methods]
impl GodotEguiTheme {
    fn new(_: &Resource) -> Self {
        Self { serialized_theme: String::new() }
    }
    #[export]
    pub fn set_theme(&mut self, _: &Resource, serialized_theme: String) {
        self.serialized_theme = serialized_theme;
    }

    pub fn get_theme(&self) -> Option<EguiTheme> {
        let raw = self.serialized_theme.to_string();
        match ron::from_str(&raw) {
            Ok(theme) => Some(theme),
            Err(err) => {
                godot_error!("{}", err);
                None
            }
        }
    }
}
