use egui::{CtxRef, Window};
use gdnative::prelude::*;
use godot_egui::GodotEgui;

#[derive(NativeClass)]
#[inherit(Control)]
pub struct GodotEguiColorTest {
    egui_test: egui_demo_lib::ColorTest,
    gui: Option<Instance<GodotEgui, Shared>>,
}

#[methods]
impl GodotEguiColorTest {
    fn new(_owner: &Control) -> Self {
        Self { egui_test: egui_demo_lib::ColorTest::default(), gui: None }
    }

    #[export]
    fn _ready(&mut self, owner: TRef<Control>) {
        godot_print!("Test node ready");
        let gui = owner
            .get_node("GodotEgui")
            .and_then(|godot_egui| unsafe { godot_egui.assume_safe() }.cast::<Control>())
            .and_then(|godot_egui| godot_egui.cast_instance::<GodotEgui>())
            .expect("Expected a `GodotEgui` child with the GodotEgui nativescript class.");

        self.gui = Some(gui.claim());
    }

    #[export]
    fn _process(&mut self, _owner: &Control, _delta: f64) {
        let gui = unsafe { self.gui.as_ref().expect("GUI initialized").assume_safe() };
        gui.map_mut(|gui, instance| {
            gui.update_ctx(instance, |ctx| self.draw(ctx));
        })
        .expect("egui error");
    }

    fn draw(&mut self, ctx: &mut CtxRef) {
        Window::new("Color Test").vscroll(true).show(ctx, |ui| {
            self.egui_test.ui(ui, &mut None);
        });
        Window::new("Settings").vscroll(true).show(ctx, |ui| {
            ctx.settings_ui(ui);
        });
    }
}
