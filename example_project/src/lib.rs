use gdnative::prelude::*;
use godot_egui::GodotEgui;

pub fn load_texture(path: &str) -> Ref<Texture> {
    let loader = ResourceLoader::godot_singleton();
    loader.load(path, "Texture", false).expect("Texture found").cast().expect("Is texture")
}

#[derive(NativeClass)]
#[inherit(gdnative::api::Control)]
pub struct GodotEguiExample {
    gui: Option<Instance<GodotEgui, Shared>>,
    elapsed_time: f64,
    counter: usize,
    checkbox: bool,
    icon_1: Ref<Texture>,
    icon_2: Ref<Texture>,
}

#[gdnative::methods]
impl GodotEguiExample {
    pub fn new(_owner: TRef<Control>) -> Self {
        Self {
            gui: None,
            counter: 0,
            checkbox: false,
            elapsed_time: 0.0,
            icon_1: load_texture("res://icon.png"),
            icon_2: load_texture("res://icon_ferris.png"),
        }
    }

    #[export]
    pub unsafe fn _ready(&mut self, owner: TRef<Control>) {
        godot_print!("Initializing godot egui");
        let gui = owner
            .get_node("GodotEgui")
            .and_then(|godot_egui| godot_egui.assume_safe().cast::<Control>())
            .and_then(|godot_egui| godot_egui.cast_instance::<GodotEgui>())
            .expect("Expected a `GodotEgui` child with the GodotEgui nativescript class.");

        self.gui = Some(gui.claim());
    }

    /// Used in the `egui::Plot` example below. Taken from the egui demo.
    fn sin_plot(&self) -> egui::plot::Line {
        let time = self.elapsed_time;
        egui::plot::Line::new(egui::plot::Values::from_explicit_callback(
            move |x| 0.5 * (2.0 * x).sin() * time.sin(),
            ..,
            512,
        ))
        .stroke(egui::Stroke::new(3.0, egui::Color32::from_rgb(200, 100, 100)))
        .name("wave")
    }

    #[export]
    pub fn _process(&mut self, _owner: TRef<Control>, delta: f64) {
        let gui = unsafe { self.gui.as_ref().expect("GUI initialized").assume_safe() };

        self.elapsed_time += delta;

        // A frame can be passed to `update` specifying background color, margin and other properties
        // You may also want to pass in `None` and draw a background using a regular Panel node instead.
        let mut frame = egui::Frame::default();
        frame.margin = egui::vec2(20.0, 20.0);

        gui.map_mut(|gui, instance| {
            // We use the `update` method here to just draw a simple UI on the central panel. If you need more
            // fine-grained control, you can use update_ctx to get access to egui's context directly.
            gui.update(instance, Some(frame), |ui| {
                ui.heading("Godot Egui - Example app");

                ui.add_space(5.0);

                if ui.button("Press me to increase counter!").clicked() {
                    self.counter += 1;
                }
                ui.label(format!("Count is: {}", self.counter));

                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.checkbox, "Is the following string awesome?");
                    if self.checkbox {
                        ui.label("It is!");
                    } else {
                        ui.label("Unfortunately, it is not.");
                    }
                });

                ui.add_space(5.0);

                ui.heading("You can even plot graphs");
                ui.add_space(5.0);

                let plot = egui::plot::Plot::new("plot_example")
                    .line(self.sin_plot())
                    .width(400.0)
                    .view_aspect(4.0 / 3.0);
                ui.add(plot);

                ui.heading("Or use your custom images");
                ui.add_space(5.0);

                // Custom textures are passed in using their `rid`. Make sure the texture resources don't get
                // deallocated for as long as egui will be using them.
                let icon_1 = unsafe { self.icon_1.assume_safe() }.get_rid();
                let icon_2 = unsafe { self.icon_2.assume_safe() }.get_rid();

                ui.horizontal(|ui| {
                    // The `rid_to_egui_texture_id` function can be used to convert an rid to an egui::TextureId
                    for _ in 0..3 {
                        ui.image(godot_egui::rid_to_egui_texture_id(icon_1), egui::vec2(64.0, 64.0));
                        ui.image(godot_egui::rid_to_egui_texture_id(icon_2), egui::vec2(64.0, 64.0));
                    }
                });
            });
        })
        .expect("Map mut should succeed");
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<GodotEguiExample>();
    godot_egui::register_classes(handle);
}

godot_init!(init);
