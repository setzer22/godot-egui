use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use egui::{Event, FullOutput};
use egui::epaint::ImageDelta;
use gdnative::api::{
    GlobalConstants, ImageTexture, InputEventMouseButton, InputEventMouseMotion, ShaderMaterial,
    VisualServer,
};

#[cfg(feature="theme_support")]
use gdnative::api::File;

use gdnative::export::{
    Export,
    hint::EnumHint
};
use gdnative::prelude::*;

/// Contains conversion tables between Godot and egui input constants (keys, mouse buttons)
pub(crate) mod enum_conversions;

/// Some helper functions and traits for godot-egui
pub mod egui_helpers;
pub mod ext;

/// Converts an egui color into a godot color
pub fn egui2color(color: egui::Color32) -> Color {
    // let as_f32 = |x| x as f32 / u8::MAX as f32;
    // Color::rgba(as_f32(c.r()), as_f32(c.g()), as_f32(c.b()), as_f32(c.a()))
    let (r, g, b, a) = egui::Rgba::from(color).to_tuple();
    Color::from_rgba(r, g, b, a)
}

/// Converts a godot color into an egui color
pub fn color2egui(color: Color) -> egui::Color32 {
    let as_u8 = |x| (x * (u8::MAX as f32)) as u8;
    egui::Color32::from_rgba_premultiplied(as_u8(color.r), as_u8(color.g), as_u8(color.b), as_u8(color.a))
    // egui::Color32::from(egui::Rgba::from_rgba_premultiplied(c.r, c.g, c.b, c.a))
}

/// Converts an u64, stored in an `egui::Texture::User` back into a Godot `Rid`.
#[allow(dead_code)]
fn u64_to_rid(x: u64) -> Rid {
    // Safety: Godot Rids should always fit in an u64, so it's safe to transmute
    unsafe { Rid::from_sys(std::mem::transmute::<u64, gdnative::sys::godot_rid>(x)) }
}

/// Converts a godot `Rid` into an `egui::TextureId`
pub fn rid_to_egui_texture_id(x: Rid) -> egui::TextureId {
    // Safety: See `u64_to_rid`
    unsafe { egui::TextureId::User(std::mem::transmute::<gdnative::sys::godot_rid, u64>(*x.sys())) }
}
#[derive(ToVariant)]
enum GodotEguiInputMode {
    None = 0,
    Input = 1,
    GuiInput = 2,
}

impl FromVariant for GodotEguiInputMode {
    fn from_variant(variant: &Variant) -> Result<Self, FromVariantError> {
        match i64::from_variant(variant)? {
            0 => Ok(GodotEguiInputMode::None),
            1 => Ok(GodotEguiInputMode::Input),
            2 => Ok(GodotEguiInputMode::GuiInput),
            _ => Err(FromVariantError::UnknownEnumVariant{
                variant: "i64".to_owned(),
                expected: &["0", "1", "2"],
            }),
        }
    }
}

impl Export for GodotEguiInputMode {
    type Hint = gdnative::export::hint::IntHint<u32>;

    fn export_info(_hint: Option<Self::Hint>) -> ExportInfo {
        Self::Hint::Enum(EnumHint::new(vec![
            "None".to_owned(),
            "Input".to_owned(),
            "GuiInput".to_owned(),
        ]))
        .export_info()
    }

}

/// Stores a canvas item, used by the visual server
struct VisualServerMesh {
    canvas_item: Rid,
}

/// Core type to draw egui-based controls in Godot.
/// The `update` or `update_ctx` methods can be used to draw a new frame.
#[derive(NativeClass)]
#[inherit(gdnative::api::Control)]
#[register_with(Self::register_properties)]
pub struct GodotEgui {
    pub egui_ctx: egui::Context,
    meshes: Vec<VisualServerMesh>,
    textures: HashMap<egui::TextureId, Ref<Texture>>,
    raw_input: Rc<RefCell<egui::RawInput>>,
    mouse_was_captured: bool,
    cursor_icon: egui::CursorIcon,

    shader_material: Option<Ref<ShaderMaterial, Shared>>,
    /// This flag will force a UI to redraw every frame.
    /// This can be used for when the UI's backend events are always changing.
    #[property(default = false)]
    reactive_update: bool,
    #[property]
    input_mode: GodotEguiInputMode,

    /// The amount of scrolled pixels per mouse wheel event
    #[property]
    scroll_speed: f32,

    /// When enabled, no texture filtering will be performed. Useful for a pixel-art style.
    #[property]
    disable_texture_filtering: bool,
    /// Pixels per point controls the render scale of the objects in egui.
    pixels_per_point: f64,
    /// The theme resource that this GodotEgui control will use.
    #[cfg(feature = "theme_support")]
    theme_path: String,
}

#[gdnative::derive::methods]
impl GodotEgui {
    fn register_properties(builder: &ClassBuilder<GodotEgui>) {
        use gdnative::export::hint::{StringHint, FloatHint, RangeHint};
        #[cfg(feature = "theme_support")]
        builder.property::<String>("EguiTheme")
            .with_getter(move |egui: &GodotEgui, _| egui.theme_path.clone())
            .with_setter(move |egui: &mut GodotEgui, _, new_val| egui.theme_path = new_val)
            .with_default("".to_owned())
            .with_hint(StringHint::File(EnumHint::new(vec!["*.ron".to_owned(), "*.eguitheme".to_owned()])))
            .done();
        builder
            .property::<f64>("pixels_per_point")
            .with_getter(move |egui: &GodotEgui, _| egui.pixels_per_point)
            .with_setter(move |egui: &mut GodotEgui, _, new_value| egui.pixels_per_point = new_value)
            .with_default(1.0)
            .with_hint(FloatHint::Range(RangeHint::new(0.01, 16.0).with_step(0.01)))
            .done();
    }
    /// Constructs a new egui node
    pub fn new(_owner: TRef<Control>) -> GodotEgui {
        GodotEgui {
            egui_ctx: Default::default(),
            meshes: vec![],
            textures: HashMap::new(),
            raw_input: Rc::new(RefCell::new(egui::RawInput::default())),
            mouse_was_captured: false,
            cursor_icon: egui::CursorIcon::Default,
            reactive_update: false,
            input_mode: GodotEguiInputMode::None,
            shader_material: None,
            scroll_speed: 20.0,
            disable_texture_filtering: false,
            pixels_per_point: 1f64,
            #[cfg(feature = "theme_support")]
            theme_path: "".to_owned(),
        }
    }

    /// Set the pixels_per_point use by `egui` to render the screen. This should be used to scale the `egui` nodes if you are using a non-standard scale for nodes in your game.
    #[export]
    pub fn set_pixels_per_point(&mut self, _owner: TRef<Control>, pixels_per_point: f64) {
        if pixels_per_point > 0f64 {
            self.pixels_per_point = pixels_per_point;
            self.egui_ctx.set_pixels_per_point(self.pixels_per_point as f32);
        } else {
            godot_error!("pixels per point must be greater than 0");
        }
    }
    /// Updates egui from the `_input` callback
    
    /// Run when this node is added to the scene tree. Runs some initialization logic, like registering any
    /// custom fonts defined as properties
    #[export]
    fn _ready(&mut self, owner: TRef<Control>) {
        match self.input_mode {
            GodotEguiInputMode::None => {
                godot_print!("GodotEgui is not accepting input");
                owner.set_mouse_filter(Control::MOUSE_FILTER_IGNORE);
                owner.set_focus_mode(Control::FOCUS_NONE);
                owner.set_process_input(false);
            },
            GodotEguiInputMode::Input => {
                godot_print!("GodotEgui is accepting input");
                owner.set_process_input(true);
                // Ignore GUI input
                owner.set_mouse_filter(Control::MOUSE_FILTER_IGNORE);
                owner.set_focus_mode(Control::FOCUS_NONE);
            },
            GodotEguiInputMode::GuiInput => {
                godot_print!("GodotEgui is accepting GUI input");
                // Ignore input
                owner.set_process_input(false);
                // Accept GUI Input
                owner.set_mouse_filter(Control::MOUSE_FILTER_PASS);
                owner.set_focus_mode(Control::FOCUS_ALL);
            }
        }
        // This decision is so that we do not have to recompile when testing the shaders.
        // TODO: Make this a build feature flag.
        self.shader_material = if let Some(material) = owner.material() {
            godot_error!("godot-egui has a material already set. This is for shader testing purposes. Clear the material if not testing shaders.");
            material.cast::<ShaderMaterial>()
        } else {
            // Create the egui shader to automatically add all the cool stuff.
            let shader = Shader::new();
            let shader_code = include_str!("egui2godot.shader");
            shader.set_code(shader_code);
            let shader = shader.into_shared();
            let shader_material = ShaderMaterial::new();
            shader_material.set_shader(shader);
            Some(shader_material.into_shared())
        };

        // Run a single dummy frame to ensure the fonts are created, otherwise egui panics
        self.egui_ctx.begin_frame(egui::RawInput::default());
        self.egui_ctx.set_pixels_per_point(self.pixels_per_point as f32);
        let FullOutput { textures_delta, .. } = self.egui_ctx.end_frame();
        for (texture_id, delta) in textures_delta.set {
            self.set_texture(texture_id, &delta)
        }
        #[cfg(feature="theme_support")]
        // We do not check if the themepath is empty. 
        if !self.theme_path.is_empty() {
            let file = File::new();
            if file.file_exists(self.theme_path.as_str()) {
                match file.open(self.theme_path.as_str(), File::READ) {
                    Ok(_) => {
                        let file_data = file.get_as_text();
                        match ron::from_str::<egui_theme::EguiTheme>(file_data.to_string().as_str()) {
                            Ok(theme) => {
                                let (style, font_definitions) = theme.extract();
                                self.egui_ctx.set_style(style);
                                self.egui_ctx.set_fonts(font_definitions);
                            }
                            Err(err) => {
                                godot_error!("Theme could not be deserialized due to: {:#?}", err);
                            }
                        }
                    }
                    Err(error) => {
                        godot_error!("{}", error);
                    }
                }
            } else {
                godot_error!("file {} does not exist", &self.theme_path)
            }
        }
    }

    /// Is used to indicate if the mouse was captured during the previous frame.
    #[export]
    pub fn mouse_was_captured(&self, _owner: TRef<Control>) -> bool {
        self.mouse_was_captured
    }

    #[export]
    pub fn _input(&mut self, owner: TRef<Control>, event: Ref<InputEvent>) {
        self.handle_godot_input(owner, event, false);
        if self.mouse_was_captured(owner) {
            // Set the input as handled by the viewport if the gui believes that is has been captured.
            unsafe { owner.get_viewport().expect("Viewport").assume_safe().set_input_as_handled() };
        }
    }

    /// Updates egui from the `_gui_input` callback
    #[export]
    pub fn _gui_input(&mut self, owner: TRef<Control>, event: Ref<InputEvent>) {
        self.handle_godot_input(owner, event, true);
        if self.mouse_was_captured(owner) {
            owner.accept_event();
        }
    }

    /// Call from the user code to pass the input event into `Egui`.
    /// `event` should be the raw `InputEvent` that is handled by `_input`, `_gui_input` and `_unhandled_input`.
    /// `is_gui_input` should be true only if this event should be processed like it was emitted from the `_gui_input` callback.
    /// # Note: If you are calling this manually, self.input_mode *MUST* be set to GodotEguiInputMode::None
    #[export]
    pub fn handle_godot_input(&mut self, owner: TRef<Control>, event: Ref<InputEvent>, is_gui_input: bool) {
        let event = unsafe { event.assume_safe() };
        let mut raw_input = self.raw_input.borrow_mut();
        let pixels_per_point = self.egui_ctx.pixels_per_point();
        // Transforms mouse positions in viewport coordinates to egui coordinates.
        let mouse_pos_to_egui = |mouse_pos: Vector2| {
            let transformed_pos = if is_gui_input {
                // Note: The `_gui_input` callback adjusts the offset before adding the event.
                mouse_pos
            } else {
                // NOTE: The egui is painted inside a control node, so its global rect offset must be taken into account.
                let offset_position = mouse_pos - owner.get_global_rect().position;
                // This is used to get the correct rotation when the root node is rotated.
                owner
                    .get_global_transform()
                    .affine_inverse()
                    .basis_xform(offset_position)
            };
            // It is necessary to translate the mouse position which refers to physical pixel position to egui's logical points
            // This is found using the inverse of current `pixels_per_point` setting.
            let points_per_pixel = 1.0 / pixels_per_point;
            egui::Pos2 { x: transformed_pos.x * points_per_pixel, y: transformed_pos.y * points_per_pixel }
        };

        if let Some(motion_ev) = event.cast::<InputEventMouseMotion>() {
            raw_input.events.push(egui::Event::PointerMoved(mouse_pos_to_egui(motion_ev.position())))
        }

        if let Some(button_ev) = event.cast::<InputEventMouseButton>() {
            if let Some(button) = enum_conversions::mouse_button_index_to_egui(button_ev.button_index()) {
                raw_input.events.push(egui::Event::PointerButton {
                    pos: mouse_pos_to_egui(button_ev.position()),
                    button,
                    pressed: button_ev.is_pressed(),
                    modifiers: Default::default(),
                })
            }

            if button_ev.is_pressed() {
                match button_ev.button_index() {
                    GlobalConstants::BUTTON_WHEEL_UP => {
                        let scroll_delta = egui::Vec2::new(0.0, 1.0) * self.scroll_speed;
                        raw_input.events.push(Event::Scroll(scroll_delta));
                    }
                    GlobalConstants::BUTTON_WHEEL_DOWN => {
                        let scroll_delta = egui::Vec2::new(0.0, -1.0) * self.scroll_speed;
                        raw_input.events.push(Event::Scroll(scroll_delta));
                    }
                    _ => {}
                }
            }
        }

        if let Some(key_ev) = event.cast::<InputEventKey>() {
            if let Some(key) = enum_conversions::scancode_to_egui(key_ev.scancode()) {
                let mods = key_ev.get_scancode_with_modifiers();
                let modifiers = egui::Modifiers {
                    ctrl: (mods & GlobalConstants::KEY_MASK_CTRL) != 0,
                    shift: (mods & GlobalConstants::KEY_MASK_SHIFT) != 0,
                    alt: (mods & GlobalConstants::KEY_MASK_ALT) != 0,
                    ..Default::default()
                };

                raw_input.events.push(egui::Event::Key { key, pressed: key_ev.is_pressed(), modifiers })
            }

            if key_ev.is_pressed() && key_ev.unicode() != 0 {
                let utf8_bytes = [key_ev.unicode() as u8];
                if let Ok(utf8) = std::str::from_utf8(&utf8_bytes) {
                    raw_input.events.push(egui::Event::Text(String::from(utf8)));
                }
            }
        }
    }

    pub fn register_godot_texture(&mut self, texture: Ref<Texture>) {
        let rid = unsafe { texture.assume_safe().get_rid() };
        self.textures.insert(rid_to_egui_texture_id(rid), texture);
    }

    fn set_texture(&mut self, texture_id: egui::TextureId, delta: &egui::epaint::ImageDelta) {
        let texture_flags = if self.disable_texture_filtering { 0 } else { Texture::FLAG_FILTER | Texture::FLAG_MIPMAPS };

        let texture = &*self.textures.entry(texture_id).or_insert_with(|| {
            assert!(delta.pos.is_none(), "when creating a new texture, the delta must be the full texture");
            let texture = ImageTexture::new();
            texture.upcast::<Texture>().into_shared()
        });
        let texture = unsafe { texture.assume_safe() };
        let texture = texture
                .cast::<ImageTexture>()
                .expect("`ImageTexture` is subclass of `Texture`");

        let delta_image = Self::image_from_delta(&delta);

        if let Some(pos) = &delta.pos {
            // partial update, blit the delta onto the texture at the correct position
            let texture_image = texture.get_data().expect("this must exist");
            let texture_image = unsafe { texture_image.assume_safe() };
             // use the entire delta image
            let blit_rect = Rect2 {
                position: Vector2::ZERO,
                size: delta_image.get_size(),
            };
            texture_image.blit_rect(delta_image, blit_rect, Vector2::new(pos[0] as _, pos[1] as _));
            texture.set_data(texture_image);
        } else {
             // full update means size changed, so we need to recreate the texture using the new image
            texture.create_from_image(delta_image, texture_flags);
        };
    }

    /// Create a Godot `Image` from an egui `ImageDelta`
    fn image_from_delta(delta: &ImageDelta) -> Ref<Image, Unique> {
        let pixels: ByteArray = match &delta.image {
            egui::ImageData::Color(egui_image) => {
                assert_eq!(
                    egui_image.width() * egui_image.height(),
                    egui_image.pixels.len(),
                    "Mismatch between texture size and texel count"
                );

                egui_image.pixels.iter().flat_map(|color| color.to_array()).collect()
            },
            egui::ImageData::Font(egui_image) => {
                assert_eq!(
                    egui_image.width() * egui_image.height(),
                    egui_image.pixels.len(),
                    "Mismatch between texture size and texel count"
                );
                // I don't really know what this is for but it was
                let gamma = 1.0 / 2.2;
                egui_image.srgba_pixels(gamma).flat_map(|a| a.to_array()).collect()
            }
        };

        let delta_image = Image::new();
        delta_image.create_from_data(
            delta.image.width() as _,
            delta.image.height() as _,
            false,
            Image::FORMAT_RGBA8,
            pixels,
        );
        delta_image
    }

    /// Paints a list of `egui::ClippedMesh` using the `VisualServer`
    fn paint_shapes(
        &mut self, owner: &Control,
        clipped_meshes: Vec<egui::ClippedPrimitive>,
        egui_texture_deltas: egui::TexturesDelta,
    ) {
        let pixels_per_point = self.egui_ctx.pixels_per_point();

        let material_rid = self
            .shader_material
            .as_ref()
            .map(|material| unsafe { material.assume_safe() }.get_rid())
            .expect("should be initialized");

        let vs = unsafe { VisualServer::godot_singleton() };

        for (id, image_delta) in &egui_texture_deltas.set {
            self.set_texture(*id, image_delta)
        }

        // Bookkeeping: Create more canvas items if needed.
        for idx in 0..clipped_meshes.len() {
            if idx >= self.meshes.len() {
                // If there's no room for this mesh, create it:
                let canvas_item = vs.canvas_item_create();
                unsafe {
                    vs.canvas_item_set_parent(canvas_item, owner.get_canvas_item());
                    vs.canvas_item_set_draw_index(canvas_item, idx as i64);
                    vs.canvas_item_clear(canvas_item);
                }
                self.meshes.push(VisualServerMesh { canvas_item /* , mesh: mesh.into_shared() */ });
            }
        }

        // Bookkeeping: Cleanup unused meshes. Pop from back to front
        for _idx in (clipped_meshes.len()..self.meshes.len()).rev() {
            let vs_mesh = self.meshes.pop().expect("This should always pop");
            unsafe {
                vs.free_rid(vs_mesh.canvas_item);
            }
        }

        assert!(
            clipped_meshes.len() == self.meshes.len(),
            "At this point, the number of canvas items should be the same as the number of egui meshes."
        );

        // Paint the meshes
        for (egui::ClippedPrimitive { clip_rect, primitive }, vs_mesh) in
            clipped_meshes.into_iter().zip(self.meshes.iter_mut())
        {
            let mesh = if let egui::epaint::Primitive::Mesh(mesh) = primitive {
                mesh
            } else {
                panic!("Expected mesh; was other primitive");
            };
            // Skip the mesh if empty, but clear the mesh if it previously existed
            if mesh.vertices.is_empty() {
                unsafe {
                    vs.canvas_item_clear(vs_mesh.canvas_item);
                }
                continue;
            }
            assert!(mesh.is_valid(), "mesh is invalid");
            
            let texture_rid = self.textures.get(&mesh.texture_id);
            if texture_rid.is_none() {
                godot_print!("{:?} does not exist", &mesh.texture_id);
                continue;
            }
            let texture_rid = unsafe {
                texture_rid.unwrap().assume_safe().get_rid()
            };

            for mut mesh in mesh.split_to_u16() {
                // First we need to get the indicies and map them to the i32 which godot understands.
                let indicies = mesh.indices.drain(0..).map(i32::from).collect::<Vec<i32>>();
                // Then we can get the indicies
                let indices = Int32Array::from_vec(indicies);
                let vertices = mesh
                    .vertices
                    .iter()
                    .map(|x| x.pos)
                    .map(|pos| Vector2::new(pos.x, pos.y))
                    .collect::<Vector2Array>();

                let uvs = mesh
                    .vertices
                    .iter()
                    .map(|x| x.uv)
                    .map(|uv| Vector2::new(uv.x, uv.y))
                    .collect::<Vector2Array>();
                let colors = mesh.vertices.iter().map(|x| x.color).map(egui2color).collect::<ColorArray>();

                unsafe {
                    vs.canvas_item_clear(vs_mesh.canvas_item);
                    vs.canvas_item_set_material(vs_mesh.canvas_item, material_rid);
                    
                    vs.canvas_item_add_triangle_array(
                        vs_mesh.canvas_item,
                        indices,
                        vertices,
                        colors,
                        uvs,
                        Int32Array::new(),
                        Float32Array::new(),
                        texture_rid,
                        -1,
                        Rid::new(),
                        false,
                        false,
                    );

                    vs.canvas_item_set_transform(
                        vs_mesh.canvas_item,
                        Transform2D::from_basis_origin(
                            Vector2::new(
                                pixels_per_point, 0.0
                            ),
                            Vector2::new(
                                0.0, pixels_per_point
                            ),
                            Vector2::new(
                                0.0, 0.0
                            )
                        ),
                    );
                    vs.canvas_item_set_clip(vs_mesh.canvas_item, true);
                    vs.canvas_item_set_custom_rect(vs_mesh.canvas_item, true, Rect2 {
                        position: Vector2::new(clip_rect.min.x, clip_rect.min.y),
                        size: Vector2::new(clip_rect.max.x - clip_rect.min.x, clip_rect.max.y - clip_rect.min.y),
                    });
                }
            
            }
            // Cleanup textures as required
            for &id in &egui_texture_deltas.free {
                self.textures.remove(&id);
            }
        }
    }

    /// Clears the screen, this can be used to cleanup the various textures and meshes that are currently being drawn to the screen from egui.
    /// # Usage Note
    /// This should only be necessary when you wish to disable an Egui node and do not wish to use the internal Godot visibility or when you wish to free canvas_item resources
    /// for memory intensive GUIs.
    #[export]
    fn clear(&mut self, _owner: TRef<Control>) {
        let vs = unsafe { VisualServer::godot_singleton() };
        for mesh in self.meshes.iter() {
            unsafe { vs.free_rid(mesh.canvas_item); }
        }
        self.meshes.clear();
    }

    /// Requests that the UI is refreshed from EGUI.
    /// Has no effect when `reactive_update` is false.
    /// ## Usage Note
    ///
    /// This should only be necessary when you have a `reactive_update` GUI that needs to respond only to changes that occur
    /// asynchronously (such as via signals) and very rarely such as a static HUD.
    ///
    /// If the UI should be updated almost every frame due to animations or constant changes with data, favor setting `reactive_update` to true instead.
    #[export]
    fn refresh(&self, _owner: TRef<Control>) {
        self.egui_ctx.request_repaint();
    }

    /// Call this to draw a new frame using a closure taking a single `egui::Context` parameter
    pub fn update_ctx(&mut self, owner: &Control, draw_fn: impl FnOnce(&mut egui::Context)) {
        assert!(owner.get_parent().is_some(), "GodotEgui must be attached in the scene tree");

        // Collect input
        let mut raw_input = self.raw_input.take();
        // Ensure that the egui context fills the entire space of the node and is adjusted accordinglly.
        let size = owner.get_rect().size;
        let points_per_pixel = (1.0 / self.pixels_per_point) as f32;
        raw_input.screen_rect =
            Some(egui::Rect::from_min_size(Default::default(), egui::Vec2::new(size.x * points_per_pixel, size.y * points_per_pixel)));

        self.egui_ctx.begin_frame(raw_input);
        
        // This ensures that while not using `reactive_update` that the UI is redrawn each frame regardless of whether the output would
        // normally request a repaint.
        if !self.reactive_update {
            self.egui_ctx.request_repaint();
        }

        draw_fn(&mut self.egui_ctx);

        // Render GUI
        let egui::FullOutput {
            platform_output,
            needs_repaint,
            shapes,
            textures_delta,
        } = self.egui_ctx.end_frame();

        // Each frame, we set the mouse_was_captured flag so that we know whether egui should be
        // consuming mouse events or not. This may introduce a one-frame lag in capturing input, but in practice it
        // shouldn't be an issue.
        self.mouse_was_captured = self.egui_ctx.is_using_pointer();

        // When we have a new cursor, we need to update the Godot side.
        if self.cursor_icon != platform_output.cursor_icon {
            self.cursor_icon = platform_output.cursor_icon;
            owner.set_default_cursor_shape(enum_conversions::mouse_cursor_egui_to_godot(self.cursor_icon).0);
        }
        // `egui_ctx` will use all the layout code to determine if there are any changes.
        // `output.needs_repaint` lets `GodotEgui` know whether we need to redraw the clipped mesh and repaint the new texture or not.
        if needs_repaint {
            let clipped_meshes = self.egui_ctx.tessellate(shapes);
            self.paint_shapes(owner, clipped_meshes, textures_delta);
        }
    }
    /// Call this to draw a new frame using a closure taking an `egui::Ui` parameter. Prefer this over
    /// `update_ctx` if the `CentralPanel` is going to be used for convenience. Accepts an optional
    /// `egui::Frame` to draw the panel background
    pub fn update(&mut self, owner: &Control, frame: Option<egui::Frame>, draw_fn: impl FnOnce(&mut egui::Ui)) {
        self.update_ctx(owner, |egui_ctx| {
            // Run user code
            egui::CentralPanel::default()
                .frame(frame.unwrap_or(egui::Frame {
                    inner_margin: egui::style::Margin::symmetric(10.0, 10.0),
                    fill: (egui::Color32::from_white_alpha(0)),
                    ..Default::default()
                }))
                .show(egui_ctx, draw_fn);
        })
    }
}

// This `Drop` is required to ensure that the VisualServerMesh RIDs are properly freed when GodotEgui is freed.
impl Drop for GodotEgui {
    fn drop(&mut self) {
        let vs = unsafe { VisualServer::godot_singleton() };
        for mesh in self.meshes.iter() {
            unsafe {
                vs.free_rid(mesh.canvas_item);
            }
        }
    }
}

/// Helper method that registers all GodotEgui `NativeClass` objects as scripts.
/// ## Note
/// This method should not be used in any library where `register_classes_as_tool` is run. Doing so may result
/// in `gdnative` errors.
pub fn register_classes(handle: InitHandle) { handle.add_class::<GodotEgui>(); }

/// Helper method that registers all GodotEgui `NativeClass` objects as tool scripts. This should **only** be
/// used when GodotEgui is to be run inside the Godot editor. ## Note
/// This method should not be used in any library where `register_classes` is run. Doing so may result in
/// `gdnative` errors.
pub fn register_classes_as_tool(handle: InitHandle) { handle.add_tool_class::<GodotEgui>(); }
