use egui::ComboBox;
use gdnative::prelude::*;
use godot_egui::{GodotEgui, ext::InputMapExt};
mod window;
use window::GodotEguiWindowExample;

mod color_test;
mod egui_demo;
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
    combox_box_value: i32,
    slider_value: f32,
    icon_1: Ref<Texture>,
    icon_2: Ref<Texture>,
    use_custom_fonts: bool,
    show_font_settings: bool,
    text_edit_text: String,

    /// Demonstrates actively manipulating the pixels_per_point setting for `egui`
    /// # Warning: This setting is very performance intensive and for demonstration purposes only.
    #[property(default = false)]
    dynamically_change_pixels_per_point: bool,
    #[property]
    handle_gui_input: bool,
    #[property]
    handle_input: bool,
}

#[gdnative::methods]
impl GodotEguiExample {
    pub fn new(_owner: TRef<Control>) -> Self {
        Self {
            gui: None,
            counter: 0,
            checkbox: false,
            combox_box_value: 0,
            slider_value: 1f32,
            elapsed_time: 0.0,
            icon_1: load_texture("res://icon.png"),
            icon_2: load_texture("res://icon_ferris.png"),
            use_custom_fonts: false,
            show_font_settings: false,
            text_edit_text: "This is a text edit!".to_owned(),
            dynamically_change_pixels_per_point: false,
            handle_gui_input: false,
            handle_input: false,
        }
    }

    #[export]
    #[gdnative::profiled]
    pub fn _ready(&mut self, owner: TRef<Control>) {
        owner.set_process_input(self.handle_input);
        if self.handle_gui_input {
            owner.set_mouse_filter(Control::MOUSE_FILTER_STOP);
        } else {
            owner.set_mouse_filter(Control::MOUSE_FILTER_IGNORE);
        }
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
    /// Updates egui from the `_input` callback
    #[export]
    #[gdnative::profiled]
    pub fn _input(&mut self, owner: TRef<Control>, event: Ref<InputEvent>) {
        let gui = unsafe { self.gui.as_ref().expect("GUI initialized").assume_safe() };
        gui.map_mut(|gui, instance| {
            gui.handle_godot_input(instance, event, false);
            if gui.mouse_was_captured(instance) {
                // Set the input as handled by the viewport if the gui believes that is has been captured.
                unsafe { owner.get_viewport().expect("Viewport").assume_safe().set_input_as_handled() };
            }
        })
        .expect("map_mut should succeed");
    }

    /// Updates egui from the `_gui_input` callback
    #[export]
    #[gdnative::profiled]
    pub fn _gui_input(&mut self, owner: TRef<Control>, event: Ref<InputEvent>) {
        let gui = unsafe { self.gui.as_ref().expect("GUI initialized").assume_safe() };
        gui.map_mut(|gui, instance| {
            gui.handle_godot_input(instance, event, true);
            if gui.mouse_was_captured(instance) {
                owner.accept_event();
            }
        })
        .expect("map_mut should succeed");
    }
    #[export]
    #[gdnative::profiled]
    pub fn _process(&mut self, _owner: TRef<Control>, delta: f64) {
        let gui = unsafe { self.gui.as_ref().expect("GUI initialized").assume_safe() };

        self.elapsed_time += delta;

        // A frame can be passed to `update` specifying background color, margin and other properties
        // You may also want to pass in `None` and draw a background using a regular Panel node instead.
        let frame = egui::Frame { margin: egui::vec2(20.0, 20.0), ..Default::default() };

        let mut should_reverse_font_priorities = false;

        gui.map_mut(|gui, instance| {
            // This resizes the window each frame based on a sine wave
            if self.dynamically_change_pixels_per_point {
                gui.set_pixels_per_point(instance, (self.elapsed_time.sin() * 0.20) + 0.8);
            }

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

                        ComboBox::from_label("This is a combo box")
                            .selected_text(format!("{}", self.combox_box_value))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.combox_box_value, 0, "0");
                                ui.selectable_value(&mut self.combox_box_value, 1, "1");
                                ui.selectable_value(&mut self.combox_box_value, 2, "2");
                                ui.selectable_value(&mut self.combox_box_value, 3, "3");
                            });

                        ui.add_space(5.0);
                        ui.label("Set the value with the slider");
                        ui.add(egui::Slider::new(&mut self.slider_value, 0.0..=100.0).text("value"));
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
                        ui.label("You can also edit text like below!");
                        ui.text_edit_multiline(&mut self.text_edit_text);
                        ui.label("And (via extension traits) capture Godot's input events (press an assigned key)");
                        let input_map = gdnative::api::InputMap::godot_singleton();
                        for action in input_map.get_actions().iter() {
                            if let Some(action) = action.try_to_string() {
                                if ui.is_action_pressed(action.as_str()) {
                                    ui.label(action.as_str());
                                }
                            }
                        }
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
    handle.add_class::<GodotEguiExample>();
    handle.add_class::<GodotEguiWindowExample>();
    handle.add_class::<color_test::GodotEguiColorTest>();
    handle.add_class::<egui_demo::GodotEguiDemoLib>();
    godot_egui::register_classes(handle);
}

godot_init!(init);
