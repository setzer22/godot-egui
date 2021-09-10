use egui_stylist::StylerState;
use gdnative::api::{FileDialog};
use gdnative::prelude::*;
use godot_egui::GodotEgui;

#[derive(NativeClass)]
#[inherit(Control)]
pub struct GodotEguiStylist {
    style: StylerState,
    godot_egui: Option<Instance<GodotEgui, Shared>>,
    file_dialog: Option<Ref<FileDialog, Shared>>,
}

#[methods]
impl GodotEguiStylist {
    fn new(_: &Control) -> Self {
        Self { style: StylerState::default(), godot_egui: None, file_dialog: None }
    }
    #[export]
    fn _ready(&mut self, owner: TRef<Control>) {
        let gui = owner
            .get_node("godot_egui")
            .and_then(|godot_egui| unsafe { godot_egui.assume_safe() }.cast::<Control>())
            .and_then(|godot_egui| godot_egui.cast_instance::<GodotEgui>())
            .expect("Expected a `GodotEgui` child with the GodotEgui nativescript class.");
        let file_dialog = owner
            .get_node("file_dialog")
            .and_then(|fd| unsafe { fd.assume_safe() }.cast::<FileDialog>())
            .expect("Expected a `FileDialog` to be present as a child with name 'file_dialog'");
        file_dialog.set_access(FileDialog::ACCESS_RESOURCES);
        file_dialog
            .connect(
                "file_selected",
                owner,
                "on_file_selected",
                VariantArray::new_shared(),
                Object::CONNECT_DEFERRED,
            )
            .expect("this should work");
        file_dialog
            .connect(
                "popup_hide",
                owner,
                "on_file_dialog_closed",
                VariantArray::new_shared(),
                Object::CONNECT_DEFERRED,
            )
            .expect("this should work");

        self.godot_egui = Some(gui.claim());
        self.file_dialog = Some(file_dialog.claim());
    }
    #[export]
    fn _process(&mut self, owner: TRef<Control>, _: f32) {
        let egui = unsafe { self.godot_egui.as_ref().expect("this must be initialized").assume_safe() };
        egui.map_mut(|gui, gui_owner| {
            gui.update_ctx(gui_owner, |ctx| {
                egui::TopBottomPanel::top("top_panel").show(ctx, |ui| self.menu_bar(owner, gui_owner, ui));
                egui::CentralPanel::default().show(ctx, |ui| self.style.ui(ui));
            });
        })
        .expect("this should work");
    }
    #[export]
    fn on_file_dialog_closed(&mut self, _: &Control) {
        godot_print!("on_file_dialog_closed");
        // owner.set_process(true);
        unsafe { self.godot_egui.as_ref().expect("should be initialized").assume_safe() }
            .map_mut(|_, o| {
                godot_print!("reenable input on `GodotEgui`");
                o.set_process_input(true);
            })
            .expect("this should work");
    }
    #[export]
    fn on_file_selected(&mut self, _: TRef<Control>, path: GodotString) {
        godot_print!("on_file_selected");
        // Do the saving or loading
        let fd = unsafe { self.file_dialog.expect("file dialog should be initialized").assume_safe() };
        if fd.mode().0 == FileDialog::MODE_OPEN_FILE {
            // TODO: Load the file
            self.style.import_theme(load_theme(path));
        } else if fd.mode().0 == FileDialog::MODE_SAVE_FILE {
            save_theme(path, self.style.export_theme());
        } else {
            godot_error!("file_dialog mode should only be MODE_SAVE_FILE or MODE_OPEN_FILE")
        }
        unsafe { self.godot_egui.as_ref().expect("should be initialized").assume_safe() }
            .map_mut(|_egui, o| {
                godot_print!("reenable input on `GodotEgui`");
                o.set_process_input(true);
            })
            .expect("this should work");
        fd.hide();
        // self.disconnect_signals(owner);
    }

    /// This creates the main menu bar that will be used by godot.
    fn menu_bar(&mut self, _: TRef<Control>, gui_owner: TRef<Control>, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            egui::menu::menu(ui, "File", |ui| {
                // TODO: Make a generic FileDialog Modal
                if ui.button("Load").clicked() {
                    let fd = unsafe { self.file_dialog.expect("file dialog should be initialized").assume_safe() };
                    fd.set_mode(FileDialog::MODE_OPEN_FILE);
                    fd.popup_centered(Vector2::new(500.0, 500.0));
                    // Push the file filters to the file dialog
                    let mut filters = StringArray::new();
                    filters.push("*.ron; Ron format".into());
                    filters.push("*.eguitheme; egui theme format".into());
                    fd.set_filters(filters);
                    godot_print!("disable input on `GodotEgui`");
                    gui_owner.set_process_input(false);
                }
                if ui.button("Save").clicked() {
                    // Option a popup to save the file to a given directory
                    let fd = unsafe { self.file_dialog.expect("file dialog should be initialized").assume_safe() };
                    fd.set_mode(FileDialog::MODE_SAVE_FILE);
                    fd.popup_centered(Vector2::new(500.0, 500.0));
                    // Push the file filters to the file dialog
                    let mut filters = StringArray::new();
                    filters.push("*.eguitheme; egui theme format".into());
                    fd.set_filters(filters);
                    godot_print!("disable input on `GodotEgui`");
                    gui_owner.set_process_input(false);
                }
            });
            egui::menu::menu(ui, "Options", |ui| {
                if ui.button("Set current theme as app theme").clicked() {
                    let ctx = ui.ctx();
                    let theme = self.style.export_theme();
                    let (style, font_definitions, ..) = theme.extract();
                    ctx.set_style(style);
                    ctx.set_fonts(font_definitions);
                }
                if ui.button("Clear settings").clicked() {
                    self.style = StylerState::default();
                    let ctx = ui.ctx();
                    ctx.set_fonts(egui::FontDefinitions::default());
                }
                if ui.button("Reset App Theme Theme").clicked() {
                    let ctx = ui.ctx();
                    ctx.set_style(egui::Style::default());
                }
            });
        });
    }
}

fn read_file(filepath: &str) -> String {
    use gdnative::api::File;
    let file = File::new();
    file.open(filepath, File::READ).unwrap_or_else(|_| panic!("{} must exist", &filepath));
    file.get_as_text().to_string()
}

pub fn load_theme(path: GodotString) -> egui_theme::EguiTheme {
    // Load the GodotEguiTheme via the ResourceLoader and then extract the EguiTheme
    let file_path = path.to_string();
    
    // We should allow for both godot resources as well as the vanilla .ron files to be published.
    let theme = {
        let file = read_file(&file_path);
        ron::from_str(&file).expect("this should load")
    };
    theme
}

pub fn save_theme(path: GodotString, theme: egui_theme::EguiTheme) {
    use gdnative::api::File;
    // First serialize the theme into ron again
    let serialized_theme = ron::to_string(&theme).expect("this should work");
    // Save it to a file
    let path = path.to_string();
    let file = File::new();
    file.open(&path, File::WRITE).unwrap_or_else(|_| panic!("{} must exist", &path));
    file.store_string(serialized_theme);
}
