# Godot Egui

An [egui](https://github.com/emilk/egui) backend for [godot-rust](https://github.com/godot-rust/godot-rust).

![Animated gif showcasing godot-egui](./resources/showcase.gif)

## Usage

These are minimal usage instructions. See the example project in `./example_project/` for a more advanced project.

First, import the `godot_egui` crate as a library in your project.

_Cargo.toml_
```toml
[dependencies]
# ...
godot_egui = { path = "path/to/this/repo/" } # You can also use git, or eventually crates.io
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

## Maturity

The project is in a very early release stage. Breaking changes may occur, but only when absolutely necessary. 

## Use cases

This integration is being used in my own project, [The Process](https://twitter.com/PlayTheProcess/status/1417774452012724226).

If you use this library and enjoy it, please feel free to submit a PR and I will add it to the list!

## Roadmap / TODO list

- [x] Initial integration and testing
- [ ] Release on crates.io
- [ ] Expose a GDScript API so this is useful even without `godot-rust`