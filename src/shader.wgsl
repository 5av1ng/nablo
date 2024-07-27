struct Uniform {
	mouse_position: vec2<f32>,
	time: f32,
	info: u32,
	position: vec2f,
	width_and_height: vec2f,
	window_xy: vec2f,
	indices_len: u32
};

@group(1) @binding(0)
var<uniform> uniforms: Uniform;

struct VertexInput {
	@location(0) position: vec3f,
	/// if is texture, we'll only use xy to stand tex_coord
	@location(1) color: vec4f,
	/// 0 = false, other = true
	@location(2) is_texture: u32,
}

struct VertexOutput {
	@builtin(position) clip_position: vec4f,
	@location(0) color: vec4f,
	@location(1) is_texture: u32,
}

@vertex
fn vs_main(
	model: VertexInput,
	@builtin(vertex_index) index: u32,
) -> VertexOutput {
	var out: VertexOutput;
	out.color = model.color;
	out.clip_position = vec4f(model.position, 1.0);
	out.is_texture = model.is_texture;
	return out;
}

@group(0)@binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
	if in.is_texture == 0u {
		return in.color;
	}else {
		return textureSample(t_diffuse, s_diffuse, in.color.xy);
	}
}