# Egui Stylist Addons

These are officially maintained godot integrations for `egui-stylist`.

At this time, it is assumed that you are familiar with godot-rust in order to use this application as you will need to build the binaries using `cargo`

In addition, to prevent editor crashes, it is a requirement that these plugins be built separately from your main game repository.
## Installation


1. Copy the "addons" directory into your godot project's "res://addons"
2. Build the needed libraries with `cargo build --lib --release`
3. Copy the resulting binaries to the following paths based on your target OS, you you wish to run the binaries out of a separate directly, you may modify the paths in the "egui_stylist_lib.gdnlib" file
    Linux: "res://addons/egui_stylist/libgodot_egui_stylist.so"
    MacOS: "res://addons/egui_stylist/libgodot_egui_stylist.dylib"
    Windows: "res://addons/egui_stylist/godot_egui_stylist.dll"
4. Enable the addons in the Godot Project Settings
Use the `cargo build --lib --release` command from the the `egui_stylist_addon` directory. Then copy the resulting binaries into `res://addons/egui_stylsit/` directory in your project.

## Egui Theme Importer addon

This import plugin is will allow you to drop a serialized `EguiTheme` file `eguitheme` and it will import the file into the Resource format that is useable by godot.

Activate this addon in the plugins menu.

## Egui Stylist Addon

This addon wraps the egui-stylist native application for convenient use from the Godot editor

Activate this addon in the plugins menu.