// use fontdue::Font;
// use once_cell::sync::Lazy;
// use std::sync::Arc;
// use crate::texture::write_texture_with_data;
// use crate::prelude::ShapeElement;
// use crate::prelude::Shape;
use wgpu::TextureView;
use nablo_shape::prelude::shape_elements::Style;
use wgpu_text::BrushBuilder;
use nablo_shape::prelude::shape_elements::DEFAULT_FONT;
use crate::texture::create_texture_with_data;
use crate::prelude::shape_elements::Rect;
use nablo_shape::prelude::ShapeMask;
use std::collections::HashMap;
use nablo_shape::prelude::Area;
use crate::ParsedShape;
use nablo_shape::prelude::shape_elements::CORRECTION;
use nablo_shape::prelude::shape_elements::EM;
use wgpu_text::TextBrush;
use wgpu_text::glyph_brush::Section as TextSection;
use wgpu_text::glyph_brush::Text as WText;
use wgpu_text::glyph_brush::ab_glyph::FontArc;
use crate::texture::create_texture;
use std::result::Result::Ok;
use crate::integrator::Output;
use wgpu::include_wgsl;
use nablo_shape::math::Vec2;
use pollster::FutureExt as _;
// use raqote::*;
use anyhow::*;

// static FONT: Lazy<Arc<Font>> = Lazy::new(|| {Arc::new(fontdue::Font::from_bytes(DEFAULT_FONT as &[u8], Default::default()).expect("loading font failed"))});

/// a struct for using wgpu
pub(crate) struct State {
	pub surface: wgpu::Surface,
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
	pub config: wgpu::SurfaceConfiguration,
	pub size: Vec2,
	pub render_pipeline: wgpu::RenderPipeline,
	pub empty_texture: WTexture,
	pub vertex_buffer: wgpu::Buffer,
	pub index_buffer: wgpu::Buffer,
	pub uniform_buffer: wgpu::Buffer,
	pub uniform_bind_group: wgpu::BindGroup,
	pub uniform_bind_group_layout: wgpu::BindGroupLayout,
	pub shader_default: wgpu::ShaderModule,
	pub fragment_shaders: HashMap<String, wgpu::ShaderModule>,
	/// None for default
	pub current_shader: Option<String>,
	// contains original image size
	pub texture_map: HashMap<String, WTexture>,
	pub brushes: Vec<TextBrush<FontArc>>,
	pub font: FontArc,
}

pub(crate) struct WTexture {
	#[allow(dead_code)]
	pub texture: wgpu::Texture,
	pub bind_group: wgpu::BindGroup,
	pub layout: wgpu::BindGroupLayout,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	position: [f32; 3],
	color: [f32; 4],
	/// 0 = false, other = true, do not find bool in wgpu VertexFormat :(.
	is_texture: u32
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniform {
	mouse_position: [f32; 2],
	/// based on second
	time: f32,
	/// you need code by your self
	info: u32,
	position: [f32; 2],
	width_and_height: [f32; 2],
	window_xy: [f32; 2],
	indices_len: u32,
}

const SHADER_STRUCT: &str = r#"
struct VertexOutput {
	@builtin(position) clip_position: vec4f,
	@location(0) color: vec4f,
	@location(1) is_texture: u32,
}

@group(0)@binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;

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
"#;


// const VERTICES: &[Vertex] = &[
// 	Vertex { position: [-1.0, 1.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], is_texture: 1 },
// 	Vertex { position: [-1.0, -1.0, 0.0], color: [0.0, 1.0, 0.0, 0.0], is_texture: 1 },
// 	Vertex { position: [1.0, -1.0, 0.0], color: [1.0, 1.0, 0.0, 0.0], is_texture: 1 },
// 	Vertex { position: [1.0, 1.0, 0.0], color: [1.0, 0.0, 0.0, 0.0], is_texture: 1 }
// ];

// const INDICES: &[u32] = &[
// 	0, 1, 2,
// 	0, 2, 3,
// ];

impl State {
	pub(crate) fn new<Window: raw_window_handle::HasRawDisplayHandle + raw_window_handle::HasRawWindowHandle>(window: &Window, size: Vec2) -> Self {
		let mut size = size;
		if size.x == 0.0{
			size.x = 640.0
		}
		if size.y == 0.0 {
			size.y = 480.0
		}

		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
			backends: wgpu::Backends::all(),
			..Default::default()
		});
		let surface = unsafe { instance.create_surface(window).expect("cant create surface") };
		let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
			compatible_surface: Some(&surface),
			..Default::default()
		}).block_on().expect("cant create adapter");

		let (device, queue) = adapter.request_device(
			&wgpu::DeviceDescriptor {
				features: wgpu::Features::empty(),
				limits: if cfg!(target_arch = "wasm32") {
					wgpu::Limits::downlevel_webgl2_defaults()
				} else {
					wgpu::Limits::default()
				},
				label: None,
			},
			None,
		).block_on().expect("cant creat device");

		let caps = surface.get_capabilities(&adapter);
		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: wgpu::TextureFormat::Rgba8UnormSrgb,
			width: size.x as u32,
			height: size.y as u32,
			present_mode: wgpu::PresentMode::Fifo,
			alpha_mode: caps.alpha_modes[0],
			view_formats: vec![],
		};
		surface.configure(&device, &config);

		let shader_default = device.create_shader_module(include_wgsl!("shader.wgsl"));

		let empty_texture = create_texture([size.x, size.y].into(), &device, &queue);


		let uniform_buffer = device.create_buffer(
			&wgpu::BufferDescriptor {
				label: Some("uniform Buffer Render"),
				size: 64,
				usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
				mapped_at_creation: false,
			}
		);

		let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			entries: &[wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::all(),
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			}],
			label: Some("uniform bind group layout"),
		});

		let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &uniform_bind_group_layout,
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: uniform_buffer.as_entire_binding(),
			}],
			label: Some("uniform bind group layout"),
		});

		let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Render Pipeline Layout"),
			bind_group_layouts: &[&empty_texture.layout, &uniform_bind_group_layout],
			push_constant_ranges: &[],
		});

		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&render_pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader_default,
				entry_point: "vs_main",
				buffers: &[desc()],
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader_default,
				entry_point: "fs_main",
				targets: &[Some(wgpu::ColorTargetState {
					format: config.format,
					blend: Some(wgpu::BlendState::ALPHA_BLENDING),
					write_mask: wgpu::ColorWrites::ALL,
				})],
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList, 
				strip_index_format: None,
				front_face: wgpu::FrontFace::Ccw,
				cull_mode: Some(wgpu::Face::Front),
				polygon_mode: wgpu::PolygonMode::Fill,
				unclipped_depth: false,
				conservative: false,
			},
			depth_stencil: None,
			multisample: wgpu::MultisampleState {
				count: 1, 
				mask: !0, 
				alpha_to_coverage_enabled: false, 
			},
			multiview: None,
		});

		let vertex_buffer = device.create_buffer(
			&wgpu::BufferDescriptor {
				label: Some("Vertex Buffer Render"),
				size: 2_u64.pow(16),
				usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
				mapped_at_creation: false,
			}
		);

		let index_buffer = device.create_buffer(
			&wgpu::BufferDescriptor {
				label: Some("Index Buffer Render"),
				size: 2_u64.pow(16),
				usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
				mapped_at_creation: false,
			}
		);

		let font = FontArc::try_from_slice(DEFAULT_FONT).unwrap();
		let brush = BrushBuilder::using_font(font.clone()).build(&device, config.width, config.height, config.format);

		Self {
			surface,
			device,
			queue,
			config,
			size,
			render_pipeline,
			empty_texture,
			vertex_buffer,
			index_buffer,
			uniform_buffer,
			uniform_bind_group,
			uniform_bind_group_layout,
			shader_default,
			texture_map: HashMap::new(),
			brushes: vec!(brush),
			font,
			fragment_shaders: HashMap::new(),
			current_shader: None
		}
	}

	pub(crate) fn resize(&mut self, new_size: Vec2) {
		if new_size.x > 0.0 && new_size.y > 0.0 {
			self.size = new_size;
			self.config.width = new_size.x as u32;
			self.config.height = new_size.y as u32;
			for brush in &self.brushes {
				brush.resize_view(self.config.width as f32, self.config.height as f32, &self.queue);
			}
			self.empty_texture = create_texture([new_size.x, new_size.y].into() , &self.device, &self.queue);
			let render_pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("Render Pipeline Layout"),
				bind_group_layouts: &[&self.empty_texture.layout, &self.uniform_bind_group_layout],
				push_constant_ranges: &[],
			});
			let module = if let Some(shader_id) = &self.current_shader {
				if let Some(shader) = self.fragment_shaders.get(shader_id) {
					shader
				}else {
					&self.shader_default
				}
			}else {
				&self.shader_default
			};
				
			let render_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
				label: Some("Render Pipeline"),
				layout: Some(&render_pipeline_layout),
				vertex: wgpu::VertexState {
					module: &self.shader_default,
					entry_point: "vs_main",
					buffers: &[desc()],
				},
				fragment: Some(wgpu::FragmentState {
					module,
					entry_point: "fs_main",
					targets: &[Some(wgpu::ColorTargetState {
						format: self.config.format,
						blend: Some(wgpu::BlendState::ALPHA_BLENDING),
						write_mask: wgpu::ColorWrites::ALL,
					})],
				}),
				primitive: wgpu::PrimitiveState {
					topology: wgpu::PrimitiveTopology::TriangleList, 
					strip_index_format: None,
					front_face: wgpu::FrontFace::Ccw,
					cull_mode: Some(wgpu::Face::Back),
					polygon_mode: wgpu::PolygonMode::Fill,
					unclipped_depth: false,
					conservative: false,
				},
				depth_stencil: None,
				multisample: wgpu::MultisampleState {
					count: 1, 
					mask: !0, 
					alpha_to_coverage_enabled: false, 
				},
				multiview: None,
			});
			self.render_pipeline = render_pipeline;
			self.surface.configure(&self.device, &self.config);
		}
	}

	pub(crate) fn draw_single_shape(&mut self, shape: ParsedShape, mouse_position: Vec2, time: f32, text_count: &mut usize, view: &TextureView) -> Result<(), wgpu::SurfaceError> {
		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("Render Texture Encoder"),
		});
		let window_size = Vec2::new(self.config.width as f32, self.config.height as f32);
		let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: Some("Render Pass Texture"),
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view,
				resolve_target: None,
				ops: wgpu::Operations {
				load: wgpu::LoadOp::Load,
				store: wgpu::StoreOp::Store,
			},
			})],
			..Default::default()
		});
		render_pass.set_pipeline(&self.render_pipeline);

		match shape {
			ParsedShape::Vertexs { vertexs, indices, clip_area, scale_factor, info } => {
				self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[Uniform {
					mouse_position: [mouse_position.x, mouse_position.y],
					position: [clip_area.left_top().x, clip_area.left_top().y],
					width_and_height: [clip_area.width_and_height().x, clip_area.width_and_height().y],
					time,
					info,
					window_xy: [window_size.x, window_size.y],
					indices_len: vertexs.len() as u32,
				}]));

				render_pass.set_bind_group(0, &self.empty_texture.bind_group, &[]);
				render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
				let mut vertexs_process = vec!();
				for vertex in vertexs {
					vertexs_process.push(Vertex {
						position: vertex.position,
						color: vertex.color,
						is_texture: 0,
					})
				}
				self.queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertexs_process));
				self.queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indices));
				let clip_area = Area::new((clip_area.area[0] + Vec2::same(1.0)) / 2.0 * window_size * scale_factor, (clip_area.area[1] + Vec2::same(1.0)) / 2.0 * window_size * scale_factor);
				let clip_area = Area::new_with_origin(window_size).cross_part(&clip_area);
				render_pass.set_scissor_rect(clip_area.area[0].x as u32, clip_area.area[0].y as u32, clip_area.width_and_height().x as u32, clip_area.width_and_height().y as u32);
				render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
				render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
				render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
			},
			ParsedShape::Text(text, style) => {
				if *text_count >= self.brushes.len() {
					self.brushes.push(BrushBuilder::using_font(self.font.clone()).build(&self.device, self.config.width, self.config.height, self.config.format))
				}
				render_pass.set_bind_group(0, &self.empty_texture.bind_group, &[]);
				let section = TextSection::default()
					.with_screen_position((style.position.x * style.scale_factor, style.position.y * style.scale_factor))
					.add_text(WText::new(text.text.leak()).with_color(style.fill.normalized()).with_scale(EM * CORRECTION * style.size.len() / 2_f32.sqrt() * style.scale_factor));
				let clip_area = Area::new_with_origin(window_size).cross_part(&Area::new(style.clip.area[0] * style.scale_factor, style.clip.area[1] * style.scale_factor));
				render_pass.set_scissor_rect(
					clip_area.area[0].x as u32, 
					clip_area.area[0].y as u32, 
					clip_area.width_and_height().x as u32, 
					clip_area.width_and_height().y as u32
				);
				self.brushes[*text_count].queue(&self.device, &self.queue, vec!(section)).unwrap();
				self.brushes[*text_count].draw(&mut render_pass);
				*text_count += 1;
			},
			ParsedShape::Image(image, style) => {
				if let Some(t) = self.texture_map.get(&image.id) {
					render_pass.set_bind_group(0, &t.bind_group, &[]);
					let mask = image.mask.unwrap_or(ShapeMask::Rect(Rect {
						width_and_height: image.size,
						..Default::default()
					}));
					let (vertexs, indices, clip_area) = mask.into_vertexs(window_size, &style);
					let clip_area = Area::new(clip_area.area[0] * style.scale_factor, clip_area.area[1] * style.scale_factor);
					let (texture_cords, _, _) = mask.into_vertexs(image.size, &Style {
						position: Vec2::ZERO,
						..style.clone()
					});
					self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[Uniform {
						mouse_position: [mouse_position.x, mouse_position.y],
						position: [clip_area.left_top().x, clip_area.left_top().y],
						width_and_height: [clip_area.width_and_height().x, clip_area.width_and_height().y],
						time,
						info: style.info,
						window_xy: [window_size.x, window_size.y],
						indices_len: vertexs.len() as u32,
					}]));
					let mut vertexs_process = vec!();
					for i in 0..vertexs.len() {
						vertexs_process.push(Vertex {
							position: vertexs[i].position,
							color: [(texture_cords[i].position[0] + 1.0) / 2.0, 1.0 - (texture_cords[i].position[1] + 1.0) / 2.0, 0.0, 0.0],
							is_texture: 1,
						})
					}
					self.queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertexs_process));
					self.queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indices));
					let clip_area = Area::new((clip_area.area[0] + Vec2::same(1.0)) / 2.0 * window_size, (clip_area.area[1] + Vec2::same(1.0)) / 2.0 * window_size);
					let clip_area = Area::new_with_origin(window_size).cross_part(&clip_area);
					render_pass.set_scissor_rect(clip_area.area[0].x as u32, clip_area.area[0].y as u32, clip_area.width_and_height().x as u32, clip_area.width_and_height().y as u32);
					render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
					render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
					render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
				}
			},
		};

		drop(render_pass);

		self.queue.submit(Some(encoder.finish()));
		Ok(())
	}

	pub(crate) fn render(&mut self, input: Output<Vec<ParsedShape>>, mouse_position: Vec2, time: f32) -> Result<(), wgpu::SurfaceError> {
		let output = self.surface.get_current_texture()?;
		let view = output.texture.create_view(&Default::default());
		// clear sections
		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("Render Texture Encoder"),
		});

		let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: Some("Render Pass"),
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view: &view,
				resolve_target: None,
				ops: wgpu::Operations {
					load: wgpu::LoadOp::Clear(wgpu::Color {
						r: input.background_color[0].powf(2.2) as f64,
						g: input.background_color[1].powf(2.2) as f64,
						b: input.background_color[2].powf(2.2) as f64,
						a: input.background_color[3].powf(2.2) as f64,
					}),
					store: wgpu::StoreOp::Store,
				},
			})],
			..Default::default()
		});
		render_pass.set_pipeline(&self.render_pipeline);
		render_pass.set_bind_group(0, &self.empty_texture.bind_group, &[]);
		drop(render_pass);

		self.queue.submit(Some(encoder.finish()));

		// draw process
		let mut text_count = 0;
		for shape in input.shapes {

			self.draw_single_shape(shape, mouse_position, time, &mut text_count, &view)?;
		}


		if text_count <= self.brushes.len() {
			let len = self.brushes.len();
			for _ in 0..(len - text_count) {
				self.brushes.pop();
			}
		}

		output.present();
		Ok(())
	}

	pub(crate) fn insert_texture(&mut self, id: String, image: crate::texture::Image) {
		self.texture_map.insert(id, create_texture_with_data(image.size, &self.device, &self.queue, image.rgba));
	}

	pub(crate) fn remove_texture(&mut self, id: &String) {
		self.texture_map.remove(id);
	}

	pub(crate) fn registrate_shader(&mut self, id: String, shader_code: String) {
		self.fragment_shaders.insert(id.clone(), self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some(&id),
			source: wgpu::ShaderSource::Wgsl(format!("{}\n{}",SHADER_STRUCT ,shader_code).into()),
		}));
	}

	pub(crate) fn remove_shader(&mut self, id: String) {
		self.fragment_shaders.remove(&id);
	}

	pub(crate) fn change_shader(&mut self, id: Option<String>) {
		self.current_shader = id;
		let render_pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Render Pipeline Layout"),
			bind_group_layouts: &[&self.empty_texture.layout, &self.uniform_bind_group_layout],
			push_constant_ranges: &[],
		});
		let module = if let Some(shader_id) = &self.current_shader {
			if let Some(shader) = self.fragment_shaders.get(shader_id) {
				shader
			}else {
				&self.shader_default
			}
		}else {
			&self.shader_default
		};
			
		let render_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&render_pipeline_layout),
			vertex: wgpu::VertexState {
				module: &self.shader_default,
				entry_point: "vs_main",
				buffers: &[desc()],
			},
			fragment: Some(wgpu::FragmentState {
				module,
				entry_point: "fs_main",
				targets: &[Some(wgpu::ColorTargetState {
					format: self.config.format,
					blend: Some(wgpu::BlendState::ALPHA_BLENDING),
					write_mask: wgpu::ColorWrites::ALL,
				})],
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList, 
				strip_index_format: None,
				front_face: wgpu::FrontFace::Ccw,
				cull_mode: Some(wgpu::Face::Back),
				polygon_mode: wgpu::PolygonMode::Fill,
				unclipped_depth: false,
				conservative: false,
			},
			depth_stencil: None,
			multisample: wgpu::MultisampleState {
				count: 1, 
				mask: !0, 
				alpha_to_coverage_enabled: false, 
			},
			multiview: None,
		});
		self.render_pipeline = render_pipeline;
		self.surface.configure(&self.device, &self.config);
	}
}

fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
	wgpu::VertexBufferLayout {
		array_stride: std::mem::size_of::<Vertex>() as u64,
		step_mode: wgpu::VertexStepMode::Vertex,
		attributes: &[
			wgpu::VertexAttribute {
				offset: 0,
				shader_location: 0,
				format: wgpu::VertexFormat::Float32x3,
			},
			wgpu::VertexAttribute {
				offset: std::mem::size_of::<[f32; 3]>() as u64,
				shader_location: 1,
				format: wgpu::VertexFormat::Float32x4,
			},
			wgpu::VertexAttribute {
				offset: std::mem::size_of::<[f32; 7]>() as u64,
				shader_location: 2,
				format: wgpu::VertexFormat::Uint32,
			}
		]
	}
}