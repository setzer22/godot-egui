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
    use_custom_fonts: bool,
    show_font_settings: bool,
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
            use_custom_fonts: false,
            show_font_settings: false,
        }
    }

    #[export]
    pub fn _ready(&mut self, owner: TRef<Control>) {
        godot_print!("Initializing godot egui");
        let gui = owner
            .get_node("GodotEgui")
            .and_then(|godot_egui| unsafe { godot_egui.assume_safe() }.cast::<Control>())
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
        let frame = egui::Frame { margin: egui::vec2(20.0, 20.0), ..Default::default() };

        let mut should_reverse_font_priorities = false;

        gui.map_mut(|gui, instance| {
            // We use the `update` method here to just draw a simple UI on the central panel. If you need more
            // fine-grained control, you can use update_ctx to get access to egui's context directly.
            gui.update_ctx(instance, /* Some(frame), */ |ctx| {
                egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
                    ui.columns(2, |columns| {
                        let ui = &mut columns[0];

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

                        // Custom textures are passed in using their `rid`. Make sure the texture resources don't
                        // get deallocated for as long as egui will be using them.
                        let icon_1 = unsafe { self.icon_1.assume_safe() }.get_rid();
                        let icon_2 = unsafe { self.icon_2.assume_safe() }.get_rid();

                        ui.horizontal(|ui| {
                            // The `rid_to_egui_texture_id` function can be used to convert an rid to an
                            // egui::TextureId
                            for _ in 0..3 {
                                ui.image(godot_egui::rid_to_egui_texture_id(icon_1), egui::vec2(64.0, 64.0));
                                ui.image(godot_egui::rid_to_egui_texture_id(icon_2), egui::vec2(64.0, 64.0));
                            }
                        });

                        let ui = &mut columns[1];

                        ui.heading("You can use custom fonts");
                        ui.label(
                            "This example registers two custom fonts. Custom fonts can be registered from the \
                             Godot Editor by setting font paths. For more control, you can also use \
                             egui::CtxRef's set_fonts method to register fonts manually.
                         \nEgui does not currently support locally overriding a font, but you can switch the \
                             global font priorities for an egui::CtxRef so that different fonts take precedence. \
                             The checkbox below will reverse the vector of fonts so that the last one, our \
                             Custom Font 2, becomes the main font.",
                        );
                        if ui.checkbox(&mut self.use_custom_fonts, "Reverse font priorities").clicked() {
                            should_reverse_font_priorities = true;
                        }

                        ui.add_space(5.0);

                        ui.heading("Icon fonts  \u{f02d}");
                        ui.add_space(5.0);
                        ui.label(
                            "By loading icon fonts, such as Fontawesome, you can easily draw small icons. Icon \
                             fonts typically use private codepoints, so there's no need to worry about \
                             priorities:\n\n \u{f091} \u{f0f3} \u{f241} \u{f0e7} \u{f0fc}",
                        );

                        ui.add_space(5.0);

                        ui.horizontal(|ui| {
                            ui.label("You can also configure font settings, check it out:");
                            if ui.button("Font settings").clicked() {
                                self.show_font_settings = true;
                            }
                        });
                    });
                });

                if self.show_font_settings {
                    let mut font_definitions = ctx.fonts().definitions().clone();
                    egui::Window::new("Settings").open(&mut self.show_font_settings).show(ctx, |ui| {
                        use egui::Widget;
                        font_definitions.ui(ui);
                        ui.fonts().texture().ui(ui);
                    });
                    ctx.set_fonts(font_definitions);
                }
            });

            if should_reverse_font_priorities {
                gui.update_ctx(instance, |ctx| {
                    let mut font_defs = ctx.fonts().definitions().clone();
                    font_defs.fonts_for_family.get_mut(&egui::FontFamily::Proportional).unwrap().reverse();
                    ctx.set_fonts(font_defs);
                })
            }
        })
        .expect("Map mut should succeed");
    }
}

fn init(handle: InitHandle) {
    handle.add_tool_class::<GodotEguiExample>();
    godot_egui::register_classes_as_tool(handle);
}

godot_init!(init);
