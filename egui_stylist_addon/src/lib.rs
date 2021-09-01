use gdnative::prelude::*;

mod stylist;

pub use stylist::GodotEguiStylist;

fn init(handle: InitHandle) {
    handle.add_tool_class::<GodotEguiStylist>();
    godot_egui::register_classes_as_tool(handle);
}


godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();