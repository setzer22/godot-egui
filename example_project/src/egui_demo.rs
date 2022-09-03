use gdnative::prelude::*;
use godot_egui::GodotEgui;

#[derive(NativeClass)]
#[inherit(Control)]
pub struct GodotEguiDemoLib {
    egui_demo: egui_demo_lib::DemoWindows,
    gui: Option<Instance<GodotEgui, Shared>>,
}

#[methods]
impl GodotEguiDemoLib {
    fn new(_owner: &Control) -> Self {
        Self { egui_demo: egui_demo_lib::DemoWindows::default(), gui: None }
    }

    #[method]
    fn _ready(&mut self, #[base] owner: TRef<Control>) {
        godot_print!("Test node ready");
        let gui = owner
            .get_node("GodotEgui")
            .and_then(|godot_egui| unsafe { godot_egui.assume_safe() }.cast::<Control>())
            .and_then(|godot_egui| godot_egui.cast_instance::<GodotEgui>())
            .expect("Expected a `GodotEgui` child with the GodotEgui nativescript class.");

        self.gui = Some(gui.claim());
    }

    #[method]
    fn _process(&mut self, _delta: f64) {
        let gui = unsafe { self.gui.as_ref().expect("GUI initialized").assume_safe() };
        gui.map_mut(|gui, instance| {
            gui.update_ctx(instance.as_ref(), |ctx| self.egui_demo.ui(ctx));
        })
        .expect("egui error");
    }
}
