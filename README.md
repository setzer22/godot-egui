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

First, import the `godot_egui` crate as a library in your project.

_Cargo.toml_
```toml
[dependencies]
# ...
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

## Running the example

Should be as simple as:

1. Run `cargo build`
2. Open `./example_project/ with the Godot Editor
3. Hit play

## Custom Fonts

Egui supports setting custom fonts out of the box: 

- https://docs.rs/egui/0.13.1/egui/struct.FontDefinitions.html
- https://docs.rs/egui/0.13.1/egui/struct.Context.html#method.set_fonts

Additionally, godot-egui supports registering custom fonts by directly from the Godot editor by exposing several script properties.
![Panel showcasing custom fonts](./resources/custom_fonts.png)

The fonts will be loaded into the `egui::FontFamily::Proportional` family in order. If you don't want egui's default fonts, the `override_default_fonts` boolean can be set so that only Custom fonts get loaded.



## Maturity

The project is in a very early release stage. Breaking changes may occur, but only when absolutely necessary. 

## Use cases

This integration is being used in my own project, [The Process](https://twitter.com/PlayTheProcess/status/1417774452012724226).

If you use this library and enjoy it, please feel free to submit a PR and I will add it to the list!

## Roadmap / TODO list

- [x] Initial integration and testing
- [ ] Release on crates.io
- [ ] Expose a GDScript API so this is useful even without `godot-rust`
