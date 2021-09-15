shader_type canvas_item;
render_mode blend_premul_alpha;

vec4 from_linear(vec4 linearRGB) {
	bvec3 cutoff = lessThanEqual(linearRGB.rgb, vec3(0.0031308));
	vec3 higher = vec3(1.055) * pow(linearRGB.rgb, vec3(1.0/2.4)) - vec3(0.055);
	vec3 lower = linearRGB.rgb * vec3(12.92);
	return vec4(mix(higher, lower, cutoff), linearRGB.a);
}


void fragment(){
	vec4 color = from_linear(COLOR);
	COLOR = texture(TEXTURE, UV) * color;
}