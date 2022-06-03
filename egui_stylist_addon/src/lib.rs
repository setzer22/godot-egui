use gdnative::prelude::*;
use gdnative::init::*;

mod stylist;
pub use stylist::GodotEguiStylist;

fn init(handle: InitHandle) {
    handle.add_tool_class::<GodotEguiStylist>();
    godot_egui::register_classes_as_tool(handle);
}

// We use the gd_egui_stylist prefix to help prevent name clashes with other namescript libraries.
// This should also alleviate any issues caused by running the addon in a separate repository that is also using `GodotEgui` This should also alleviate issues caused by running addons and tool scripts in the same repo, since it's possible to get them if you aren't careful.
godot_gdnative_init!(_ as gd_egui_stylist_gdnative_init);
godot_nativescript_init!(init as gd_egui_stylist_nativescript_init);
godot_gdnative_terminate!(_ as gd_egui_stylist_gdnative_terminate);
