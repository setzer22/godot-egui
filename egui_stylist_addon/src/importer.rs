use gdnative::api::{EditorImportPlugin, File, ResourceSaver};
use gdnative::prelude::*;
use godot_egui::GodotEguiTheme;

#[derive(NativeClass, Copy, Clone, Default)]
#[inherit(EditorImportPlugin)]
#[user_data(gdnative::nativescript::user_data::Aether<Self>)]
pub struct GodotEguiThemeImporter;
#[methods]
impl GodotEguiThemeImporter {
    fn new(_: &EditorImportPlugin) -> Self {
        Self {}
    }

    #[export]
    pub fn get_importer_name(&self, _: &EditorImportPlugin) -> String {
        "Egui Theme Importer".to_owned()
    }
    #[export]
    pub fn get_visible_name(&self, _: &EditorImportPlugin) -> String {
        "Egui Theme".to_owned()
    }
    #[export]
    pub fn get_preset_count(&self, _: &EditorImportPlugin) -> i64 {
        0i64
    }
    #[export]
    pub fn get_import_options(&self, _: &EditorImportPlugin, _: i64) -> VariantArray {
        VariantArray::new_shared()
    }
    #[export]
    pub fn get_preset_name(&self, _: &EditorImportPlugin, _: i64) -> String {
        "".to_owned()
    }
    #[export]
    pub fn get_recognized_extensions(&self, _: &EditorImportPlugin) -> VariantArray {
        let va = VariantArray::new();
        va.push("eguitheme".to_owned());
        va.into_shared()
    }
    #[export]
    pub fn get_save_extension(&self, _: &EditorImportPlugin) -> String {
        "tres".to_owned()
    }
    #[export]
    pub fn get_resource_type(&self, _: &EditorImportPlugin) -> String {
        "Resource".to_owned()
    }
    /// This function contains the importer logic to greatly simplify importing the resource.
    #[export]
    pub fn import(
        &self, owner: &EditorImportPlugin, source_file: GodotString, save_path: GodotString, _options: Dictionary,
        _r_platform_variants: VariantArray, _r_gen_files: VariantArray,
    ) -> u32 {
        let file = File::new();

        file.open(source_file.clone(), File::READ)
            .unwrap_or_else(|_| panic!("{} must exist", source_file.clone()));
        let serialized_theme = file.get_as_text().to_string();
        let resource = GodotEguiTheme { serialized_theme }.emplace().into_base();

        let save_path = format!("{}.{}", save_path, self.get_save_extension(owner));
        let saver = ResourceSaver::godot_singleton();
        match saver.save(save_path, resource, ResourceSaver::FLAG_REPLACE_SUBRESOURCE_PATHS) {
            Ok(()) => 0u32,
            Err(err) => err as u32,
        }
    }
}
