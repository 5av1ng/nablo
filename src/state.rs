use fontdue::Font;
use once_cell::sync::Lazy;
use std::sync::Arc;
use crate::texture::write_texture_with_data;
use crate::prelude::ShapeElement;
use crate::prelude::Shape;
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

static FONT: Lazy<Arc<Font>> = Lazy::new(|| {Arc::new(fontdue::Font::from_bytes(DEFAULT_FONT as &[u8], Default::default()).expect("loading font failed"))});

/// a struct for using wgpu
pub(crate) struct State {
	pub surface: wgpu::Surface,
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
	pub config: wgpu::SurfaceConfiguration,
	pub size: Vec2,
	pub render_pipeline: wgpu::RenderPipeline,
	pub empty_texture: WTexture,
	pub vertex_buffer: Vec<wgpu::Buffer>,
	pub index_buffer: Vec<wgpu::Buffer>,
	pub shader: wgpu::ShaderModule,
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

const VERTICES: &[Vertex] = &[
	Vertex { position: [-1.0, 1.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], is_texture: 1 },
	Vertex { position: [-1.0, -1.0, 0.0], color: [0.0, 1.0, 0.0, 0.0], is_texture: 1 },
	Vertex { position: [1.0, -1.0, 0.0], color: [1.0, 1.0, 0.0, 0.0], is_texture: 1 },
	Vertex { position: [1.0, 1.0, 0.0], color: [1.0, 0.0, 0.0, 0.0], is_texture: 1 }
];

const INDICES: &[u32] = &[
	0, 1, 2,
	0, 2, 3,
];

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

		let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

		let empty_texture = create_texture([size.x, size.y].into(), &device, &queue);

		let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Render Pipeline Layout"),
			bind_group_layouts: &[&empty_texture.layout],
			push_constant_ranges: &[],
		});

		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&render_pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main",
				buffers: &[desc()],
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
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

		let vertex_buffer = vec!(device.create_buffer(
			&wgpu::BufferDescriptor {
				label: Some(&format!("Vertex Buffer Render")),
				size: 2_u64.pow(16),
				usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
				mapped_at_creation: false,
			}
		));

		let index_buffer = vec!(device.create_buffer(
			&wgpu::BufferDescriptor {
				label: Some(&format!("Index Buffer Render")),
				size: 2_u64.pow(16),
				usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
				mapped_at_creation: false,
			}
		));

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
			shader,
			texture_map: HashMap::new(),
			brushes: vec!(brush),
			font
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
				bind_group_layouts: &[&self.empty_texture.layout],
				push_constant_ranges: &[],
			});
			let render_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
				label: Some("Render Pipeline"),
				layout: Some(&render_pipeline_layout),
				vertex: wgpu::VertexState {
					module: &self.shader,
					entry_point: "vs_main",
					buffers: &[desc()],
				},
				fragment: Some(wgpu::FragmentState {
					module: &self.shader,
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

	pub(crate) fn render(&mut self, input: Output<Vec<ParsedShape>>) -> Result<(), wgpu::SurfaceError> {
		let output = self.surface.get_current_texture()?;
		let view = output.texture.create_view(&Default::default());
		let window_size = Vec2::new(self.config.width as f32, self.config.height as f32);
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
						r: (input.background_color[0] as f64 / 255.0).powf(2.2),
						g: (input.background_color[1] as f64 / 255.0).powf(2.2),
						b: (input.background_color[2] as f64 / 255.0).powf(2.2),
						a: (input.background_color[3] as f64 / 255.0).powf(2.2),
					}),
					store: wgpu::StoreOp::Store,
				},
			})],
			..Default::default()
		});
		render_pass.set_pipeline(&self.render_pipeline);
		render_pass.set_bind_group(0, &self.empty_texture.bind_group, &[]);
		drop(render_pass);

		// draw process
		let mut i = 0;
		let mut text_count = 0;
		for shape in input.shapes {
			if i >= self.vertex_buffer.len() {
				let vertex_buffer = self.device.create_buffer(
					&wgpu::BufferDescriptor {
						label: Some(&format!("Vertex Buffer Render")),
						size: 2_u64.pow(16),
						usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
						mapped_at_creation: false,
					}
				);

				let index_buffer = self.device.create_buffer(
					&wgpu::BufferDescriptor {
						label: Some(&format!("Index Buffer Render")),
						size: 2_u64.pow(16),
						usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
						mapped_at_creation: false,
					}
				);
				self.vertex_buffer.push(vertex_buffer);
				self.index_buffer.push(index_buffer);
			}

			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render Pass Texture"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
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
				ParsedShape::Vertexs { vertexs, indices, clip_area } => {
					render_pass.set_bind_group(0, &self.empty_texture.bind_group, &[]);
					let mut vertexs_process = vec!();
					for vertex in vertexs {
						vertexs_process.push(Vertex {
							position: vertex.position,
							color: vertex.color,
							is_texture: 0,
						})
					}
					self.queue.write_buffer(&self.vertex_buffer[i], 0, bytemuck::cast_slice(&vertexs_process));
					self.queue.write_buffer(&self.index_buffer[i], 0, bytemuck::cast_slice(&indices));
					let clip_area = Area::new((clip_area.area[0] + Vec2::same(1.0)) / 2.0 * window_size, (clip_area.area[1] + Vec2::same(1.0)) / 2.0 * window_size);
					let clip_area = Area::new_with_origin(window_size).cross_part(&clip_area);
					render_pass.set_scissor_rect(clip_area.area[0].x as u32, clip_area.area[0].y as u32, clip_area.width_and_height().x as u32, clip_area.width_and_height().y as u32);
					render_pass.set_vertex_buffer(0, self.vertex_buffer[i].slice(..));
					render_pass.set_index_buffer(self.index_buffer[i].slice(..), wgpu::IndexFormat::Uint32);
					render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
					i = i + 1;
				},
				ParsedShape::Text(text, style) => {
					if text_count >= self.brushes.len() {
						self.brushes.push(BrushBuilder::using_font(self.font.clone()).build(&self.device, self.config.width, self.config.height, self.config.format))
					}
					render_pass.set_bind_group(0, &self.empty_texture.bind_group, &[]);
					let section = TextSection::default()
						.with_screen_position((style.position.x, style.position.y))
						.add_text(WText::new(text.text.leak()).with_color(style.fill.normalized()).with_scale(EM * CORRECTION * style.size.len() / 2_f32.sqrt()));
					let clip_area = Area::new_with_origin(window_size).cross_part(&style.clip);
					render_pass.set_scissor_rect(clip_area.area[0].x as u32, clip_area.area[0].y as u32, clip_area.width_and_height().x as u32, clip_area.width_and_height().y as u32);
					self.brushes[text_count].queue(&self.device, &self.queue, vec!(section)).unwrap();
					self.brushes[text_count].draw(&mut render_pass);
					text_count = text_count + 1;
				},
				ParsedShape::Image(image, style) => {
					if let Some(t) = self.texture_map.get(&image.id) {
						render_pass.set_bind_group(0, &t.bind_group, &[]);
						let mask = image.mask.unwrap_or(ShapeMask::Rect(Rect {
							width_and_height: image.size,
							..Default::default()
						}));
						let (vertexs, indices, clip_area) = mask.into_vertexs(window_size, &style);
						let (texture_cords, _, _) = mask.into_vertexs(image.size, &Style {
							position: Vec2::ZERO,
							..style.clone()
						});
						let mut vertexs_process = vec!();
						for i in 0..vertexs.len() {
							vertexs_process.push(Vertex {
								position: vertexs[i].position,
								color: [(texture_cords[i].position[0] + 1.0) / 2.0, 1.0 - (texture_cords[i].position[1] + 1.0) / 2.0, 0.0, 0.0],
								is_texture: 1,
							})
						}
						self.queue.write_buffer(&self.vertex_buffer[i], 0, bytemuck::cast_slice(&vertexs_process));
						self.queue.write_buffer(&self.index_buffer[i], 0, bytemuck::cast_slice(&indices));
						let clip_area = Area::new((clip_area.area[0] + Vec2::same(1.0)) / 2.0 * window_size, (clip_area.area[1] + Vec2::same(1.0)) / 2.0 * window_size);
						let clip_area = Area::new_with_origin(window_size).cross_part(&clip_area);
						render_pass.set_scissor_rect(clip_area.area[0].x as u32, clip_area.area[0].y as u32, clip_area.width_and_height().x as u32, clip_area.width_and_height().y as u32);
						render_pass.set_vertex_buffer(0, self.vertex_buffer[i].slice(..));
						render_pass.set_index_buffer(self.index_buffer[i].slice(..), wgpu::IndexFormat::Uint32);
						render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
						i = i + 1;
					}
				},
			};

			drop(render_pass);
		}

		if i <= self.vertex_buffer.len() {
			let len = self.vertex_buffer.len();
			for _ in 0..(len - i) {
				self.vertex_buffer.pop();
				self.index_buffer.pop();
			}
		}

		if text_count <= self.brushes.len() {
			let len = self.brushes.len();
			for _ in 0..(len - text_count) {
				self.brushes.pop();
			}
		}

		self.queue.submit(Some(encoder.finish()));
		output.present();

		Ok(())
	}

	pub(crate) fn soft_render(&mut self, input: Output<Vec<Shape>>) -> Result<(), wgpu::SurfaceError> {
		let output = self.surface.get_current_texture()?;
		let view = output.texture.create_view(&Default::default());
		let window_size = Vec2::new(self.config.width as f32, self.config.height as f32);

		fn parse_shape(shape: &ShapeMask, style: &Style) -> tiny_skia::Path {
			let mut pb = tiny_skia::PathBuilder::new();

			match shape {
				ShapeMask::Circle(inner) => {
					pb.push_circle(inner.radius, inner.radius, inner.radius);
				},
				ShapeMask::Rect(inner) => {
					if inner.rounding == Vec2::ZERO {
						pb.push_rect(tiny_skia::Rect::from_ltrb(0.0,0.0,inner.width_and_height.x, inner.width_and_height.y).unwrap());
					}else {
						let magic_number = 4.0 / 3.0 * (2.0_f32.sqrt() - 1.0);
						pb.move_to(inner.width_and_height.x, inner.width_and_height.y - inner.rounding.y);
						pb.cubic_to(inner.width_and_height.x, inner.width_and_height.y + (magic_number - 1.0) * inner.rounding.y,
							inner.width_and_height.x + (magic_number - 1.0) * inner.rounding.x, inner.width_and_height.y,
							inner.width_and_height.x - inner.rounding.x, inner.width_and_height.y);

						pb.line_to(inner.rounding.x, inner.width_and_height.y);
						pb.cubic_to((1.0 - magic_number) * inner.rounding.x, inner.width_and_height.y,
							0.0, inner.width_and_height.y + (magic_number - 1.0) *inner.rounding.y,
							0.0, inner.width_and_height.y - inner.rounding.y);

						pb.line_to(0.0, inner.rounding.y);
						pb.cubic_to(0.0, (1.0 - magic_number) * inner.rounding.y,
							(1.0 - magic_number) * inner.rounding.x, 0.0,
							inner.rounding.x, 0.0);

						pb.line_to(inner.width_and_height.x - inner.rounding.x, 0.0);
						pb.cubic_to(inner.width_and_height.x + (magic_number - 1.0) * inner.rounding.x, 0.0,
							inner.width_and_height.x, (1.0 - magic_number) * inner.rounding.y,
							inner.width_and_height.x, inner.rounding.y);
						pb.line_to(inner.width_and_height.x, inner.width_and_height.y - inner.rounding.y);
						pb.close();
					}
				},
				ShapeMask::CubicBezier(inner) => {
					pb.move_to(inner.points[0].x, inner.points[0].y);
					pb.cubic_to(inner.points[1].x, inner.points[1].y, inner.points[2].x, inner.points[2].y, inner.points[3].x, inner.points[3].y);
					if inner.if_close {
						pb.close();
					}
				},
				ShapeMask::Line(inner) => {
					pb.cubic_to(0.0, 0.0, inner.x, inner.y, inner.x, inner.y);
				},
				ShapeMask::Polygon(inner) => {
					let mut started = false;
					for point in &inner.points {
						if !started {
							pb.move_to(point.x, point.y);
							started = true;
							continue;
						}
						pb.line_to(point.x, point.y);
					}
					pb.close();
				},
			}
			let path = pb.finish().unwrap();

			let transform = tiny_skia::Transform::identity()
				.post_translate(style.position.x, style.position.y)
				.post_translate(-style.transform_origin.x, -style.transform_origin.y)
				.post_rotate(style.rotate / std::f32::consts::PI * 180.0)
				.post_scale(style.size.x, style.size.y)
				.post_translate(style.transform_origin.x, style.transform_origin.y);

			path.transform(transform).unwrap()
		}

		let mut pixmap = tiny_skia::Pixmap::new(window_size.x as u32, window_size.y as u32).unwrap();

		for shape in input.shapes {
			let mut mask = tiny_skia::Mask::new(window_size.x as u32, window_size.y as u32).unwrap();
			let mut pb = tiny_skia::PathBuilder::new();
			pb.push_rect(tiny_skia::Rect::from_ltrb(shape.style.clip.area[0].x, shape.style.clip.area[0].y, shape.style.clip.area[1].x, shape.style.clip.area[1].y).unwrap());
			let clip = pb.finish().unwrap();
			mask.fill_path(&clip, tiny_skia::FillRule::EvenOdd, true, Default::default());
			match shape.shape {
				ShapeElement::Text(inner) => {
					let transform = tiny_skia::Transform::identity()
						.post_translate(shape.style.position.x, shape.style.position.y)
						.post_translate(-shape.style.transform_origin.x, -shape.style.transform_origin.y)
						.post_rotate(shape.style.rotate / std::f32::consts::PI * 180.0)
						.post_scale(shape.style.size.x, shape.style.size.y)
						.post_translate(shape.style.transform_origin.x, shape.style.transform_origin.y);
					let em = EM * shape.style.size.len() / 2.0_f32.sqrt() * CORRECTION;
					let fonts = &[FONT.clone()];
					let mut layout = fontdue::layout::Layout::new(fontdue::layout::CoordinateSystem::PositiveYDown);
					layout.reset(&Default::default());
					layout.append(fonts, &fontdue::layout::TextStyle::new(&inner.text, em, 0));
					let glyphs = layout.glyphs();
					for glyph in glyphs {
						let (metrics, image_data) = FONT.rasterize(glyph.parent, em);
						if image_data.len() == 0 {
							continue
						}
						let mut data = vec!();
						for image_data in image_data {
							data.push(((image_data as f32 * shape.style.fill[0] as f32) / 255.0) as u8);
							data.push(((image_data as f32 * shape.style.fill[1] as f32) / 255.0) as u8);
							data.push(((image_data as f32 * shape.style.fill[2] as f32) / 255.0) as u8);
							data.push(((image_data as f32 * shape.style.fill[3] as f32) / 255.0) as u8);
						}
						let slice = data.as_slice();
						let map = tiny_skia::PixmapRef::from_bytes(slice, metrics.width as u32, metrics.height as u32).unwrap();
						pixmap.draw_pixmap(glyph.x as i32, glyph.y as i32, map, &Default::default(), transform, Some(&mask))
					}
				},
				ShapeElement::Image(_) => {},
				_ => {
					let path = parse_shape(&shape.shape.into_mask(), &shape.style);
					let mut paint = tiny_skia::Paint::default();
					paint.set_color_rgba8(shape.style.fill[0], shape.style.fill[1], shape.style.fill[2], shape.style.fill[3]);
					paint.anti_alias = false;

					pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Default::default(), Some(&mask));

					if shape.style.stroke_width > 0.0 && shape.style.stroke_color[3] != 0 {
						paint.set_color_rgba8(shape.style.fill[0], shape.style.fill[1], shape.style.fill[2], shape.style.fill[3]);
						pixmap.stroke_path(&path, &paint, &tiny_skia::Stroke { width: shape.style.stroke_width, ..Default::default() }, Default::default(), Some(&mask));
					}
				}
			}
		};

		write_texture_with_data(window_size, &self.empty_texture.texture, &self.queue, pixmap.data());

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
						r: (input.background_color[0] as f64 / 255.0).powf(2.2),
						g: (input.background_color[1] as f64 / 255.0).powf(2.2),
						b: (input.background_color[2] as f64 / 255.0).powf(2.2),
						a: (input.background_color[3] as f64 / 255.0).powf(2.2),
					}),
					store: wgpu::StoreOp::Store,
				},
			})],
			..Default::default()
		});
		render_pass.set_pipeline(&self.render_pipeline);
		render_pass.set_bind_group(0, &self.empty_texture.bind_group, &[]);

		self.queue.write_buffer(&self.vertex_buffer[0], 0, bytemuck::cast_slice(&VERTICES));
		self.queue.write_buffer(&self.index_buffer[0], 0, bytemuck::cast_slice(&INDICES));
		render_pass.set_vertex_buffer(0, self.vertex_buffer[0].slice(..));
		render_pass.set_index_buffer(self.index_buffer[0].slice(..), wgpu::IndexFormat::Uint32);
		render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
		drop(render_pass);

		self.queue.submit(Some(encoder.finish()));
		output.present();

		Ok(())
	}

	pub(crate) fn insert_texture(&mut self, id: String, image: crate::texture::Image) {
		self.texture_map.insert(id, create_texture_with_data(image.size, &self.device, &self.queue, image.rgba));
	}

	pub(crate) fn remove_texture(&mut self, id: &String) {
		self.texture_map.remove(id);
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