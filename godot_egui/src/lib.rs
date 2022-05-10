use std::cell::RefCell;
use std::rc::Rc;

use egui::{Event, FontDefinitions};
use gdnative::api::{
    File, GlobalConstants, ImageTexture, InputEventMouseButton, InputEventMouseMotion, VisualServer,
};
use gdnative::export::hint::{EnumHint, StringHint};
use gdnative::prelude::*;

/// Contains conversion tables between Godot and egui input constants (keys, mouse buttons)
pub(crate) mod enum_conversions;

/// Some helper functions and traits for godot-egui
pub mod egui_helpers;

/// Converts an egui color into a godot color
pub fn egui2color(c: egui::Color32) -> Color {
    let as_f32 = |x| x as f32 / u8::MAX as f32;
    Color::from_rgba(as_f32(c.r()), as_f32(c.g()), as_f32(c.b()), as_f32(c.a()))
}

/// Converts a godot color into an egui color
pub fn color2egui(c: Color) -> egui::Color32 {
    let as_u8 = |x| (x * (u8::MAX as f32)) as u8;
    egui::Color32::from_rgba_premultiplied(as_u8(c.r), as_u8(c.g), as_u8(c.b), as_u8(c.a))
}

/// Converts an u64, stored in an `egui::Texture::User` back into a Godot `Rid`.
fn u64_to_rid(x: u64) -> Rid {
    // Safety: Godot Rids should always fit in an u64, so it's safe to transmute
    unsafe { Rid::from_sys(std::mem::transmute::<u64, gdnative::sys::godot_rid>(x)) }
}

/// Converts a godot `Rid` into an `egui::TextureId`
pub fn rid_to_egui_texture_id(x: Rid) -> egui::TextureId {
    // Safety: See `u64_to_rid`
    unsafe { egui::TextureId::User(std::mem::transmute::<gdnative::sys::godot_rid, u64>(*x.sys())) }
}

/// Stores a canvas item, used by the visual server
struct VisualServerMesh {
    canvas_item: Rid,
}

/// Holds the data for the egui main texture in Godot memory.
struct SyncedTexture {
    texture_version: Option<u64>,
    godot_texture: Ref<ImageTexture>,
}

/// Core type to draw egui-based controls in Godot.
/// The `update` or `update_ctx` methods can be used to draw a new frame.
#[derive(NativeClass)]
#[inherit(gdnative::api::Control)]
#[register_with(register_properties)]
pub struct GodotEgui {
    pub egui_ctx: egui::Context,
    meshes: Vec<VisualServerMesh>,
    main_texture: SyncedTexture,
    raw_input: Rc<RefCell<egui::RawInput>>,
    mouse_was_captured: bool,

    /// If set to true, egui's default fonts will be ignored. You can set `custom_fonts` instead.
    #[property]
    override_default_fonts: bool,
    /// Custom font paths. If set, they will get loaded into egui during _ready
    custom_fonts: [Option<String>; 5],

    /// The amount of scrolled pixels per mouse wheel event
    #[property]
    scroll_speed: f32,

    /// Whether or not this egui should call set_input_as_handled after receiving a mouse event.
    #[property]
    consume_mouse_events: bool,

    /// When enabled, no texture filtering will be performed. Useful for a pixel-art style.
    #[property]
    disable_texture_filtering: bool,
}

fn register_properties(builder: &ClassBuilder<GodotEgui>) {
    for i in 0..5 {
        builder
            .property::<String>(&format!("custom_font_{}", i + 1))
            .with_getter(move |x: &GodotEgui, _| x.custom_fonts[i].as_ref().cloned().unwrap_or_default())
            .with_setter(move |x: &mut GodotEgui, _, new_val| x.custom_fonts[i] = Some(new_val))
            .with_default("".to_owned())
            .with_hint(StringHint::File(EnumHint::new(vec!["*.ttf".to_owned(), "*.otf".to_owned()])))
            .done();
    }
}

#[gdnative::derive::methods]
impl GodotEgui {
    /// Constructs a new egui node
    pub fn new(_owner: TRef<Control>) -> GodotEgui {
        GodotEgui {
            egui_ctx: Default::default(),
            meshes: vec![],
            main_texture: SyncedTexture {
                texture_version: None,
                godot_texture: ImageTexture::new().into_shared(),
            },
            raw_input: Rc::new(RefCell::new(egui::RawInput::default())),
            mouse_was_captured: false,
            override_default_fonts: false,
            custom_fonts: [None, None, None, None, None],
            scroll_speed: 20.0,
            consume_mouse_events: true,
            disable_texture_filtering: false,
        }
    }

    /// Run when this node is added to the scene tree. Runs some initialization logic, like registering any
    /// custom fonts defined as properties
    #[godot]
    fn _ready(&mut self) {
        // Run a single dummy frame to ensure the fonts are created, otherwise egui panics
        self.egui_ctx.begin_frame(egui::RawInput::default());
        let _ = self.egui_ctx.end_frame();

        // This is where "res://" points to
        //let mut font_defs = self.egui_ctx.fonts().definitions().clone();
        let mut font_defs = FontDefinitions::default(); // TODO(bromeon): changed from egui 0.15->0.18; correct?

        if self.override_default_fonts {
            font_defs.families.get_mut(&egui::FontFamily::Proportional).unwrap().clear()
        }

        for font_path in self
            .custom_fonts
            .iter()
            .filter(|x| x.as_ref().map(|x| !x.is_empty()).unwrap_or(false))
            .map(|x| x.as_ref().unwrap())
        {
            let font_file = gdnative::api::File::new();
            match font_file.open(font_path, File::READ) {
                Ok(_) => {
                    let file_data = font_file.get_buffer(font_file.get_len());
                    let file_data = file_data.read().as_slice().to_owned();
                    let font_data = egui::FontData::from_owned(file_data);
                    font_defs.font_data.insert(font_path.to_owned(), font_data);
                    font_defs
                        .families
                        .get_mut(&egui::FontFamily::Proportional)
                        .unwrap()
                        .push(font_path.to_owned());
                }
                Err(error) => {
                    godot_error!("GodotEgui could not load a custom font file: {:?}", error);
                }
            }
        }

        self.egui_ctx.set_fonts(font_defs);
    }

    fn maybe_set_mouse_input_as_handled(&self, owner: TRef<Control>) {
        if self.mouse_was_captured && self.consume_mouse_events {
            unsafe { owner.get_viewport().expect("Viewport").assume_safe().set_input_as_handled() }
        }
    }

    /// Callback to listen for input. Translates input back to egui events.
    #[godot]
    fn _input(&mut self, #[base] owner: TRef<Control>, event: Ref<InputEvent>) {
        let event = unsafe { event.assume_safe() };
        let mut raw_input = self.raw_input.borrow_mut();

        // Transforms mouse positions in viewport coordinates to egui coordinates.
        // NOTE: The egui is painted inside a control node, so its global rect offset must be taken into account
        let mouse_pos_to_egui = |mouse_pos: Vector2| {
            let transformed_pos = mouse_pos - owner.get_global_rect().position;
            egui::Pos2 { x: transformed_pos.x, y: transformed_pos.y }
        };

        if let Some(motion_ev) = event.cast::<InputEventMouseMotion>() {
            self.maybe_set_mouse_input_as_handled(owner);
            raw_input.events.push(egui::Event::PointerMoved(mouse_pos_to_egui(motion_ev.position())))
        }

        if let Some(button_ev) = event.cast::<InputEventMouseButton>() {
            self.maybe_set_mouse_input_as_handled(owner);
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
                    alt: (mods & GlobalConstants::KEY_ALT) != 0,
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

    /// Paints a list of `egui::ClippedMesh` using the `VisualServer`
    fn paint_shapes(
        &mut self, owner: &Control, clipped_meshes: Vec<egui::ClippedPrimitive>,
        egui_texture: &egui::TextureHandle,
    ) {
        let vs = unsafe { VisualServer::godot_singleton() };

        // Sync egui's texture to our Godot texture, only when needed
        if self.main_texture.texture_version != Some(egui_texture.version) {
            let pixels: ByteArray =
                egui_texture.pixels.iter().map(|alpha| [255u8, 255u8, 255u8, *alpha]).flatten().collect();

            let image = Image::new();
            image.create_from_data(
                egui_texture.width as i64,
                egui_texture.height as i64,
                false,
                Image::FORMAT_RGBA8,
                pixels,
            );

            self.main_texture.texture_version = Some(egui_texture.version);

            let new_tex = ImageTexture::new();
            // NOTE: It's important for the texture to be non-repeating.
            // This is because the egui texture has a full white pixel at (0,0), which is used by many opaque
            // shapes. When using the default flags, blending + wrapping end up lowering the alpha of
            // the pixel at (0,0)
            let flags =
                if self.disable_texture_filtering { 0 } else { Texture::FLAG_FILTER | Texture::FLAG_MIPMAPS };
            new_tex.create_from_image(image, flags);
            self.main_texture.godot_texture = new_tex.into_shared();
        }

        let egui_texture_rid = unsafe { self.main_texture.godot_texture.assume_safe() }.get_rid();

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

            let texture_rid = match mesh.texture_id {
                egui::TextureId::Managed(_id) => egui_texture_rid, // TODO(bromeon): should we use id in Managed?
                egui::TextureId::User(id) => u64_to_rid(id),
            };

            // Safety: Transmuting from Vec<u32> to Vec<i32> should be safe as long as indices don't overflow.
            // If the index array overflows we will just get an OOB and crash which is fine.
            #[allow(clippy::unsound_collection_transmute)]
            let indices = Int32Array::from_vec(unsafe { std::mem::transmute::<_, Vec<i32>>(mesh.indices) });
            let vertices = mesh
                .vertices
                .iter()
                .map(|x| x.pos)
                .map(|pos| Vector2::new(pos.x, pos.y))
                .collect::<Vector2Array>();

            let uvs =
                mesh.vertices.iter().map(|x| x.uv).map(|uv| Vector2::new(uv.x, uv.y)).collect::<Vector2Array>();
            let colors = mesh.vertices.iter().map(|x| x.color).map(egui2color).collect::<ColorArray>();

            unsafe {
                vs.canvas_item_clear(vs_mesh.canvas_item);
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

                vs.canvas_item_set_clip(vs_mesh.canvas_item, true);
                vs.canvas_item_set_custom_rect(vs_mesh.canvas_item, true, Rect2 {
                    position: Vector2::new(clip_rect.min.x, clip_rect.min.y),
                    size: Vector2::new(clip_rect.max.x - clip_rect.min.x, clip_rect.max.y - clip_rect.min.y),
                });
            }
        }
    }

    /// Call this to draw a new frame using a closure taking a single `egui::Context` parameter
    pub fn update_ctx(&mut self, owner: &Control, draw_fn: impl FnOnce(&mut egui::Context)) {
        assert!(owner.get_parent().is_some(), "GodotEgui must be attached in the scene tree");

        // Collect input
        let mut raw_input = self.raw_input.take();
        let size = owner.get_rect().size;
        raw_input.screen_rect =
            Some(egui::Rect::from_min_size(Default::default(), egui::Vec2::new(size.x, size.y)));

        self.egui_ctx.begin_frame(raw_input);

        draw_fn(&mut self.egui_ctx);

        // Render GUI
        let egui::FullOutput { shapes, .. } = self.egui_ctx.end_frame();

        // Each frame, we set the mouse_was_captured flag so that we know whether egui should be
        // consuming mouse events or not. This may introduce a one-frame lag in capturing input, but in practice it
        // shouldn't be an issue.
        self.mouse_was_captured = self.egui_ctx.is_using_pointer();

        let clipped_meshes = self.egui_ctx.tessellate(shapes);
        self.paint_shapes(owner, clipped_meshes, &self.egui_ctx.texture());
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

    pub fn mouse_was_captured(&self) -> bool { self.mouse_was_captured }
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
