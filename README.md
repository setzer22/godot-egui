# Godot Egui

[![Latest version](https://img.shields.io/crates/v/godot_egui.svg)](https://crates.io/crates/godot_egui)
[![Documentation](https://docs.rs/godot_egui/badge.svg)](https://docs.rs/godot_egui)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

An [egui](https://github.com/emilk/egui) backend for [godot-rust](https://github.com/godot-rust/godot-rust).

![Animated gif showcasing godot-egui](./resources/showcase.gif)

## Rationale

Godot has a perfectly valid GUI system, so why `egui`? Here are my personal reasons:

- Simplicity: No need to connect signals or manage complex scene graphs with dozens of tiny inter-related scripts. GUI logic is centralized. Everything is always up-to-date.
- Better defaults: It currently takes a lot of effort to create a visually consistent UI theme for Godot (margins, font sizes, colors...). In contrast, `egui`'s widgets only rely on a small set of themable properties.
- Customizability: Creating new widgets with `egui` is [far more simple](https://github.com/emilk/egui/blob/master/egui_demo_lib/src/apps/demo/toggle_switch.rs). 
- Data driven: The immediate-mode paradigm fits a data-driven style of development: Your data is the source of truth, and the GUI is derived from it by navigating and updating the data itself.
- IDE Support: Rust has excellent IDE support. Static typing helps you ensure your data model and their associated GUIs always stay in sync.

## Usage

These are minimal usage instructions. See the example project in `./example_project/` for a more advanced project.

### Configuration

First, import the `godot_egui` crate as a library in your project.

_Cargo.toml_
```toml
[dependencies]
# ...
egui = "0.13"
godot_egui = "0.1.1"
```

Next, register the custom Godot classes declared in `godot_egui`:

_Somewhere in your lib.rs_
```rust
fn init(handle: InitHandle) {
    godot_egui::register_classes(handle);
}

godot_init!(init);
```

You will also need to create a `.gdns` script and attach it to a `Control`-derived node.

_GodotEgui.gdns_
```ini
[gd_resource type="NativeScript" load_steps=2 format=2]

[ext_resource path="res://godot_egui.gdnlib" type="GDNativeLibrary" id=1]

[resource]
resource_name = "GodotEgui"
class_name = "GodotEgui"
library = ExtResource( 1 )
```

### Retreving Godot Input

In order to handle input, `GodotEgui` exposes the `handle_godot_input` and `mouse_was_captured` functions that can be used to pass input events from your node into `GodotEgui`.

To handle input from `_input` or `_unhandled_input` use the following:

```rust
#[method]
// fn _unhandled_input(&mut self, owner: TRef<Control>, event: Ref<InputEvent>) also works.
fn _input(&mut self, #[base] owner: TRef<Control>, event: Ref<InputEvent>) {
    let gui = unsafe { self.gui.as_ref().expect("GUI initialized").assume_safe() };
    gui.map_mut(|gui, instance| {
        gui.handle_godot_input(instance, event, false);
        if gui.mouse_was_captured() {
            // Set the input as handled by the viewport if the gui believes that is has been captured.
            unsafe { owner.get_viewport().expect("Viewport").assume_safe().set_input_as_handled() };
        }
    }).expect("map_mut should succeed");
}

```

To handle input from `_gui_input` use the following:

```rust
#[method]
fn _gui_input(&mut self, #[base] owner: TRef<Control>, event: Ref<InputEvent>) {
    let gui = unsafe { self.gui.as_ref().expect("GUI initialized").assume_safe() };
    gui.map_mut(|gui, instance| {
        gui.handle_godot_input(instance, event, true);
        if gui.mouse_was_captured() {
            // `_gui_input` uses accept_event() to stop the propagation of events.
            owner.accept_event();
        }
    }).expect("map_mut should succeed");
}
```

### Important

`GodotEgui` translates the infomration from Godot's [`InputEvent`](https://docs.godotengine.org/en/stable/classes/class_inputevent.html#class-inputevent) class into the input format that `egui` expects. `GodotEgui` does not make any assumptions about how or when this data is input, it only translates and forwards the information to `egui`.

When determining how to handle the events, please refer to Godot's input propagation rules (see [the official documentation](https://docs.godotengine.org/en/stable/tutorials/inputs/inputevent.html) for more details), this cannot be easily translated perfectly for every architecture. As such, it is 


**Note**: Regarding using `_gui_input` with `GodotEgui`. You will need to ensure that you are properly configuring `Control.mouse_filter` and that you understand how this works with `egui`.

- `Control::MOUSE_FILTER_STOP` - Consumes all input events. and it will not propagate up.
- `Control::MOUSE_FILTER_PASS` - Consumes all input events., but will allow unhandled events to propagate to the parent node.
- `Control::MOUSE_FILTER_IGNORE` - Does not consume input events.

**Note regarding `Control::MOUSE_FILTER_PASS`**:

The Godot engine assumption appears to be that most Controls that accept input will not overlap with other sibling nodes. As each `GodotEgui` constitutes it's own `egui::Context` (i.e. application), it will need to process any events that occur inside the `Control.rect.size`. This can lead to scenarios where input may be expected to propagate (i.e. `egui` does not detect that the mouse position overlaps any `egui::Widget` types) but does not.

The only current workaround is show in the [GodotEguiWindowExample](example_project/GodotEguiWindowExample.tscn) where UIs are parented to eachother in the order or priority they should response to the events.

### Drawing the `egui::Ui`

Finally, get a reference to that node as a `RefInstance<GodotEgui, Shared>` in your code and do this to draw the GUI using:

```rust
let gui : RefInstance<GodotEgui, Shared> = ...;
gui.map_mut(|gui, instance| {
    gui.update(instance, None, |ui| {
        ui.label("Hello world!");
    });
})
```

The draw code needs to be run constantly, so you should call it from a `_process` callback or similar.

### Getting Godot Input from `egui::Ui`

GodotEgui includes the `ext` module along with several extension traits that can be used to access the Godot Input data from `egui::Ui`.

These can be imported by using `use godot_egui::ext::*` or by importing only the extension traits that you need for your project.

#### Example

```rust
use godot_egui::ext::InputMapExt;
gui.update(instance, None, |ui| {
    ui.label("Is 'ui_up' pressed?");
    if ui.is_action_pressed("ui_up") {
        ui.label("Yes!");
    }
});
```

## Running the example

Should be as simple as:

1. Run `cargo build`
2. Open `./example_project/ with the Godot Editor
3. Hit play

## Themes

Godot Egui supports themes based on the `egui-theme` module of [egui-stylist](https://github.com/jacobsky/egui-stylist/) and ships with two addons that can be used to create export and automatically import themes into a godot usable format.

Themes are an intermediate format that will allow for more flexible serialization of `egui::Style` and `egui::FontDefinitions` structs and are fully configurable and support Serialization and Deserialization via any Serialization format that you can support.

For more information on how to create your first theme please see the [egui-stylist addon's readme](./egui_stylist_addon/README.md).

**Can I use my pre-existing Godot Themes with Egui?**

Unfortunately, this is not supported.

This is due to egui and Godot taking very different approaches to UI skinning. Godot has very fine grained themes and tools for modifying how widgets draw themselves while egui has far simplier sets of rules for coloring windows and widgets. As a result, there's no good way to map between the two.

## Custom Fonts

Custom fonts have been removed now that [Theme](#themes) support has been added.

## Maturity

The project is in a very early release stage. Breaking changes may occur, but only when absolutely necessary. 

## Use cases

This integration is being used in my own project, [The Process](https://twitter.com/PlayTheProcess/status/1417774452012724226).

If you use this library and enjoy it, please feel free to submit a PR and I will add it to the list!

## Roadmap / TODO list

- [x] Initial integration and testing
- [x] Release on crates.io
- [x] Enable usage as an editor plugin
- [x] Theme editor [#5](https://github.com/setzer22/godot-egui/issues/5)
- [ ] Expose a GDScript API so this is useful even without `godot-rust`
