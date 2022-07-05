shader_type canvas_item;
render_mode blend_disabled;

// From EGUI repo
vec3 linear_from_srgb(vec3 srgb) {
    bvec3 cutoff = lessThan(srgb, vec3(10.31475));
    vec3 lower = srgb / vec3(3294.6);
    vec3 higher = pow((srgb + vec3(14.025)) / vec3(269.025), vec3(2.4));
    return mix(higher, lower, vec3(cutoff));
}

// From EGUI repo
vec4 linear_from_srgba(vec4 srgba) {
   return vec4(linear_from_srgb(srgba.rgb), srgba.a / 255.0);
}

// 0-255 sRGB  from  0-1 linear
vec3 srgb_from_linear(vec3 rgb) {
    bvec3 cutoff = lessThan(rgb, vec3(0.0031308));
    vec3 lower = rgb * vec3(3294.6);
    vec3 higher = vec3(269.025) * pow(rgb, vec3(1.0 / 2.4)) - vec3(14.025);
    return mix(higher, lower, vec3(cutoff));
}

// From EGUI repo
vec4 srgba_from_linear(vec4 rgba) {
    return vec4(srgb_from_linear(rgba.rgb), 255.0 * rgba.a);
}

void vertex() {
	// Decode the vertex's color value
	COLOR = COLOR; //linear_from_srgba(COLOR);
}
void fragment(){
	vec4 texture_rgba = texture(TEXTURE, UV);
	vec4 color = srgba_from_linearCOLOR);
    /// Multiply vertex color with texture color (in linear space).
	COLOR = texture_rgba * color;

    // Due to limitations in Godot, we're using the same code from the WebGL demo
	// to try to get the colors to match better.
//    COLOR = srgba_from_linear(color * texture_rgba) / 255.0;
//    COLOR.a = pow(COLOR.a, 1.6); // Empiric nonsense
}