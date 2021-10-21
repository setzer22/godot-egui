//! These Extension traits extend `egui::Ui` to have easy access to the Godot Input singleton. 
//! This module helps bridge the gap between `egui`'s input events and allows you to leverage Godot's Input singleton
//! directly in `egui` 
//! 
//! To use these extension methods, import the needed extensions into your `GodotEgui` project where you need the traits.
use gdnative::prelude::*;
use gdnative::api::Resource;
use gdnative::api::input::{MouseMode, CursorShape};

impl InputMapExt for egui::Ui {}
impl MouseKeyboardInputExt for egui::Ui {}
impl MobileInputExt for egui::Ui {}
impl JoypadInputExt for egui::Ui {}

/// An extension that can be used to add access to Godot's `Input` singleton's methods directly from any other object.
/// This trait specifically implements input handling exclusive to Mobile Devices.
pub trait MobileInputExt {
    #[inline]
    fn get_accelerometer(&self) -> Vector3 {
        Input::godot_singleton().get_accelerometer()
    }
    #[inline]
    fn get_gyroscope(&self) -> Vector3 {
        Input::godot_singleton().get_gyroscope()
    }
    #[inline]
    fn get_magnetometer(&self) -> Vector3 {
        Input::godot_singleton().get_magnetometer()
    }
    #[inline]
    fn get_gravity(&self) -> Vector3 {
        Input::godot_singleton().get_gravity()
    }
    #[inline]
    fn vibrate_handheld(&self, duration_ms: i64) {
        Input::godot_singleton().vibrate_handheld(duration_ms)
    }
}

/// An extension that can be used to add access to Godot's `Input` singleton's methods directly from any other object.
/// This trait specifically implements input handling joypads.
pub trait JoypadInputExt {
    #[inline]
    fn is_joy_button_pressed(&self, device: i64, button: i64) -> bool {
        Input::godot_singleton().is_joy_button_pressed(device, button)
    }
    #[inline]
    fn is_joy_known(&self, device: i64) -> bool {
        Input::godot_singleton().is_joy_known(device)
    }
    #[inline]
    fn get_connected_joypads(&self) -> VariantArray<Shared> {
        Input::godot_singleton().get_connected_joypads()
    }
    #[inline]
    fn get_joy_axis(&self, device: i64, axis: i64) -> f64 {
        Input::godot_singleton().get_joy_axis(device, axis)
    }
    #[inline]
    fn get_joy_axis_index_from_string(&self,axis: impl Into<GodotString>) -> i64 {
        Input::godot_singleton().get_joy_axis_index_from_string(axis)
    }
    #[inline]
    fn get_joy_axis_string(&self, axis_index: i64) -> GodotString {
        Input::godot_singleton().get_joy_axis_string(axis_index)
    }
    #[inline]
    fn get_joy_button_index_from_string(&self, button: impl Into<GodotString>) -> i64 {
        Input::godot_singleton().get_joy_button_index_from_string(button)
    }
    #[inline]
    fn get_joy_button_string(&self, button_index: i64) -> GodotString {
        Input::godot_singleton().get_joy_button_string(button_index)
    }
    #[inline]
    fn get_joy_guid(&self, device: i64) -> GodotString {
        Input::godot_singleton().get_joy_guid(device)
    }
    #[inline]
    fn get_joy_name(&self, device: i64) -> GodotString {
        Input::godot_singleton().get_joy_name(device)
    }
    #[inline]
    fn get_joy_vibration_duration(&self, device: i64) -> f64 {
        Input::godot_singleton().get_joy_vibration_duration(device)
    }
    #[inline]
    fn get_joy_vibration_strength(&self, device: i64) -> Vector2 {
        Input::godot_singleton().get_joy_vibration_strength(device)
    }
    #[inline]
    fn joy_connection_changed(&self,device: i64, connected: bool, name: impl Into<GodotString>, guid: impl Into<GodotString>) {
        Input::godot_singleton().joy_connection_changed(device, connected, name, guid)
    }
    #[inline]
    fn remove_joy_mapping(&self, guid: impl Into<GodotString>) {
        Input::godot_singleton().remove_joy_mapping(guid)
    }
    #[inline]
    fn start_joy_vibration(&self,device: i64, weak_magnitude: f64, strong_magnitude: f64, duration: f64) {
        Input::godot_singleton().start_joy_vibration(device, weak_magnitude, strong_magnitude, duration)
    }
    #[inline]
    fn stop_joy_vibration(&self, device: i64) {
        Input::godot_singleton().stop_joy_vibration(device)
    }
}

/// An extension that can be used to add access to Godot's `Input` singleton's methods directly from any other object.
/// This trait specifically implements input handling for mouse and keyboard.
pub trait MouseKeyboardInputExt {
    #[inline]
    fn last_mouse_speed(&self) -> egui::Vec2 {
        let speed = Input::godot_singleton().get_last_mouse_speed();
        egui::vec2(speed.x, speed.y)
    }
    #[inline]
    fn mouse_button_mask(&self) -> i64 {
        Input::godot_singleton().get_mouse_button_mask()
    }
    #[inline]
    fn mouse_mode(&self) -> MouseMode {
        Input::godot_singleton().get_mouse_mode()
    }
    #[inline]
    fn is_key_pressed(&self, scancode: i64) -> bool {
        Input::godot_singleton().is_key_pressed(scancode)
    }
    #[inline]
    fn is_mouse_button_pressed(&self, button: i64) -> bool {
        Input::godot_singleton().is_mouse_button_pressed(button)
    }
    #[inline]
    fn set_custom_mouse_cursor(&self, image: impl AsArg<Resource>, shape: i64, hotspot: Vector2) {
        Input::godot_singleton().set_custom_mouse_cursor(image, shape, hotspot)
    }
    #[inline]
    fn set_mouse_mode(&self, mode: i64) {
        Input::godot_singleton().set_mouse_mode(mode)
    }
    #[inline]
    fn warp_mouse_position(&self, to: Vector2) {
        Input::godot_singleton().warp_mouse_position(to)
    }
    
}

/// An extension that can be used to add access to Godot's `Input` singleton's methods directly from any other object.
/// This trait specifically implements general input using functionality based around the `InputMap`'s actions
pub trait InputMapExt {
    #[inline]
    fn cursor_shape(&self) -> CursorShape {
        Input::godot_singleton().get_current_cursor_shape()
    }
    #[inline]
    fn action_strength(&self, action: impl Into<GodotString>) -> f64 {
        Input::godot_singleton().get_action_strength(action)
    }
    #[inline]
    fn is_action_pressed(&self, action: impl Into<GodotString>) -> bool {
        Input::godot_singleton().is_action_pressed(action)
    }
    #[inline]
    fn is_action_just_pressed(&self, action: impl Into<GodotString>) -> bool {
        Input::godot_singleton().is_action_just_pressed(action)
    }
    #[inline]
    fn is_action_just_released(&self, action: impl Into<GodotString>) -> bool {
        Input::godot_singleton().is_action_just_released(action)
    }
    #[inline]
    fn parse_input_event(&self, event: impl AsArg<InputEvent>) {
        Input::godot_singleton().parse_input_event(event);
    }
    #[inline]
    fn set_default_cursor_shape(&self, shape: CursorShape) {
        Input::godot_singleton().set_default_cursor_shape(shape.0)
    }
    #[inline]
    fn set_use_accumulated_input(&self, enable: bool) {
        Input::godot_singleton().set_use_accumulated_input(enable)
    }
}
