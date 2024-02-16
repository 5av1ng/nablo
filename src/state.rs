use crate::PASSWORD;
use sw_composite::muldiv255;
use fontdue::Font;
use crate::texture::create_texture;
use wgpu::util::DeviceExt;
use nablo_shape::shape::shape_elements::Shape;
use nablo_shape::prelude::shape_elements::Rect;
use std::collections::HashMap;
use nablo_shape::prelude::ShapeMask;
use nablo_shape::prelude::shape_elements::EM;
use std::result::Result::Ok;
use std::f32::consts::PI;
use euclid::Angle;
use euclid::Vector2D;
use euclid::Transform2D;
use nablo_shape::prelude::shape_elements::Style;
use nablo_shape::prelude::ShapeElement;
use crate::integrator::ShapeOutput;
use crate::integrator::Output;
use wgpu::include_wgsl;
use std::iter;
use nablo_shape::math::Vec2;
use winit::window::Window;
use pollster::FutureExt as _;
use raqote::*;
use anyhow::*;
use crate::texture::Image as NabloImage;
use rayon::prelude::*;

/// a struct for using wgpu
pub(crate) struct State {
	pub surface: wgpu::Surface,
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
	pub config: wgpu::SurfaceConfiguration,
	pub size: Vec2,
	pub render_pipeline: wgpu::RenderPipeline,
	pub diffuse_texture: wgpu::Texture,
	pub diffuse_bind_group: wgpu::BindGroup,
	pub vertex_buffer: wgpu::Buffer,
	pub index_buffer: wgpu::Buffer,
	pub shader: wgpu::ShaderModule,
}


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	position: [f32; 3],
	tex_coords: [f32; 2],
}

const VERTICES: &[Vertex] = &[
	Vertex { position: [-1.0, 1.0, 0.0], tex_coords: [0.0, 0.0], },
	Vertex { position: [-1.0, -1.0, 0.0], tex_coords: [0.0, 1.0], },
	Vertex { position: [1.0, -1.0, 0.0], tex_coords: [1.0, 1.0], },
	Vertex { position: [1.0, 1.0, 0.0], tex_coords: [1.0, 0.0], }
];

const INDICES: &[u32] = &[
	0, 1, 2,
	0, 2, 3,
];

const MOVE_DOWN_LETTER: [char; 36] = ['a','c','e','g','m','n','o','p','q','r','s','u','v','w','x','y','z','α','γ','ε','η','ι','κ','μ','ν','ο','π','ρ','σ','τ','υ','χ','ω','>','<','~'];

impl State {
	pub(crate) fn new(window: &Window) -> Self {
		let mut size = Vec2::new(window.inner_size().width as f32, window.inner_size().height as f32);
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
			format: caps.formats[0],
			width: size.x as u32,
			height: size.y as u32,
			present_mode: wgpu::PresentMode::Fifo,
			alpha_mode: caps.alpha_modes[0],
			view_formats: vec![],
		};
		surface.configure(&device, &config);

		let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

		let (diffuse_texture,texture_bind_group_layout , diffuse_bind_group) = create_texture([size.x, size.y].into(), &device, &queue);

		let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Render Pipeline Layout"),
			bind_group_layouts: &[&texture_bind_group_layout],
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

		let vertex_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Vertex Buffer"),
				contents: bytemuck::cast_slice(VERTICES),
				usage: wgpu::BufferUsages::VERTEX,
			}
		);

		let index_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Index Buffer"),
				contents: bytemuck::cast_slice(INDICES),
				usage: wgpu::BufferUsages::INDEX,
			}
		);

		Self {
			surface,
			device,
			queue,
			config,
			size,
			render_pipeline,
			diffuse_texture,
			diffuse_bind_group,
			vertex_buffer,
			index_buffer,
			shader,
		}
	}

	pub(crate) fn resize(&mut self, new_size: Vec2) {
		if new_size.x > 0.0 && new_size.y > 0.0 {
			self.size = new_size;
			self.config.width = new_size.x as u32;
			self.config.height = new_size.y as u32;
			let (diffuse_texture, texture_bind_group_layout, diffuse_bind_group) = create_texture([new_size.x, new_size.y].into() , &self.device, &self.queue);
			self.diffuse_texture = diffuse_texture;
			self.diffuse_bind_group = diffuse_bind_group;
			let render_pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("Render Pipeline Layout"),
				bind_group_layouts: &[&texture_bind_group_layout],
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

	pub(crate) fn render(&mut self, input: &Output<ShapeOutput>, font: &[Font; 4], image_memory: &HashMap<String, NabloImage>) -> Result<(), wgpu::SurfaceError> {
		cfg_if::cfg_if! {
			if #[cfg(target_arch = "wasm32")] {
				let mut binding = draw(&input.shapes, self.size, font, image_memory);
				let shapes = binding.get_data_u8_mut();
				shapes.par_chunks_mut(4).for_each(|inside| {
					let temp = inside[0];
					inside[0] = inside[2];
					inside[2] = temp;
				});
			}else {
				let binding = draw(&input.shapes, self.size, font, image_memory);
				let shapes = binding.get_data_u8();
			}
		} 

		let output = self.surface.get_current_texture()?;
		let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("Render Encoder"),
		});
		let texture_size = wgpu::Extent3d {
			width: self.size.x as u32,
			height: self.size.y as u32,
			depth_or_array_layers: 1,
		};
		self.queue.write_texture(wgpu::ImageCopyTexture {
			texture: &self.diffuse_texture,
			mip_level: 0,
			origin: wgpu::Origin3d::ZERO,
			aspect: wgpu::TextureAspect::All,
		}, &shapes, wgpu::ImageDataLayout {
			offset: 0,
			bytes_per_row: Some(self.size.x as u32 * 4),
			rows_per_image: Some(self.size.y as u32),
		}, texture_size);
		// self.queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertexs));
		// self.queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indices));

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
		render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
		render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
		render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
		render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
		drop(render_pass);

		self.queue.submit(iter::once(encoder.finish()));
		output.present();

		Ok(())
	}
}

fn draw(input: &ShapeOutput, window_size: Vec2, font: &[Font; 4], image_memory: &HashMap<String, NabloImage>) -> DrawTarget {
	let mut dt = DrawTarget::new(window_size.x as i32, window_size.y as i32);
	for shape in &input.shapes {
		handle_shape(&mut dt, &shape.shape, &shape.style, font, image_memory);
	};
	dt
}

fn handle_shape(dt: &mut DrawTarget, shape: &ShapeElement , style: &Style, fonts: &[Font; 4], image_memory: &HashMap<String, NabloImage>) {
	dt.push_clip_rect(IntRect {
		min: euclid::Point2D::new(style.clip.area[0].x as i32, style.clip.area[0].y as i32),
		max: euclid::Point2D::new(style.clip.area[1].x as i32, style.clip.area[1].y as i32)
	});
	let transform = Transform2D::identity()
		.then_translate(Vector2D::new(style.position.x, style.position.y))
		.then_translate(Vector::new(-style.transform_origin.x, -style.transform_origin.y))
		.then_rotate(Angle { radians: style.rotate })
		.then_scale(style.size.x, style.size.y)
		.then_translate(Vector::new(style.transform_origin.x, style.transform_origin.y));
	if let ShapeElement::Text(t) = shape {
		if t.text.is_empty() {
			dt.pop_clip();
			return;
		}
		let em = EM * style.size.len() / 2.0_f32.sqrt(); 
		let mut x = style.position.x;
		let mut line_counter = 0.0;
		// let y_m = match fonts[0].vertical_line_metrics(em) {
		// 	None => fonts[0].metrics('M', em).height as f32,
		// 	Some(t) => t.new_line_size
		// };
		for i in 0..utf8_slice::len(&t.text) {
			let y;
			let width;
			let text = utf8_slice::slice(&t.text, i, i + 1).chars().next().unwrap();
			if text == '\n' {
				line_counter = line_counter + 1.0;
				x = style.position.x;
				continue;
			}
			let (metrics, bitmap) = if t.text_style.is_bold {
				if t.text_style.is_italic {
					fonts[3].rasterize(text, em)
				}else {
					fonts[1].rasterize(text, em)
				}
			}else {
				if t.text_style.is_italic {
					fonts[2].rasterize(text, em)
				}else {
					fonts[0].rasterize(text, em)
				}
			};
			// width = metrics.width as f32;
			// y = metrics.ymin as f32 + line_counter * em + style.position.y;
			// println!("x: {}, y: {}", x, y);
			if (text >= '一' && text <= '龥') || text == PASSWORD  {
				width = em * 0.8 * 1.5;
				y = style.position.y + line_counter * em;
			}else {
				width = em * 0.8;
				y = if MOVE_DOWN_LETTER.contains(&text)  {
					style.position.y + em * 0.21
				}else if text == ',' || text == '.' || text == '_' {
					style.position.y + em * 0.67
				}else if text == '=' {
					style.position.y + em * 0.3
				}else if text == '-' {
					style.position.y + em * 0.45
				}else if text == '+' {
					style.position.y + em * 0.25
				}else if text == '\n' {
					line_counter = line_counter + 1.0;
					x = style.position.x;
					continue;
				}else {
					style.position.y
				} + line_counter * em;
			}
			let data: Vec<u32> = bitmap.into_par_iter().map(|input| {
				let input = (input as f32 * style.fill[3] as f32 / 255.0) as u8;
				if input == 0 {
					0
				}else {
					(input as u32) << 24 | 
					muldiv255(input as u32, style.fill[0] as u32) << 16 | 
					muldiv255(input as u32, style.fill[1] as u32) << 8 | 
					muldiv255(input as u32, style.fill[2] as u32)
				}
			}).collect();
			dt.draw_image_at(x, y, &Image {
				width: metrics.width as i32,
				height: metrics.height as i32,
				data: &data
			}, &DrawOptions::new());
			// dt.draw_text(font, em, utf8_slice::slice(&t.text, i, i + 1), Point::new(x, style.position.y), 
			// &Source::Solid(SolidSource::from_unpremultiplied_argb(style.fill[3], style.fill[0], style.fill[1], style.fill[2])), &DrawOptions::new());
			x = x + width;
		};
		dt.pop_clip();
		return;
	}

	if let ShapeElement::Image(image) = shape {
		let mask = image.mask.clone().unwrap_or_else(|| ShapeMask::Rect(Rect { width_and_height: image.get_area(style).width_and_height(), rounding: Vec2::same(0.0) }));
		let path = parse_shape(&mask, &transform);
		if let Some(t) = image_memory.get(&image.id) {
			let data: Vec<u32> = t.rgba.clone().into_par_iter().chunks(4).map(|input| {
				if input[3] == 0 {
					0
				}else {
					(input[3] as u32) << 24 | 
					muldiv255(input[3] as u32, input[0] as u32) << 16 | 
					muldiv255(input[3] as u32, input[1] as u32) << 8 | 
					muldiv255(input[3] as u32, input[2] as u32)
				}
			}).collect();
			let transform = Transform2D::identity()
				.then_translate(Vector::new(style.transform_origin.x, style.transform_origin.y))
				.then_rotate(Angle { radians: style.rotate })
				.then_scale(style.size.x, style.size.y)
				.then_translate(Vector::new(-style.transform_origin.x, -style.transform_origin.y))
				.then_translate(Vector2D::new(-style.position.x, -style.position.y))
				.then_scale(t.size.x / image.size.x, t.size.y / image.size.y);
			dt.fill(&path, &Source::Image(Image {
				width: t.size.x as i32,
				height: t.size.y as i32,
				data: &data
			}, ExtendMode::Repeat, FilterMode::Bilinear, transform), &DrawOptions::new());
		}
		dt.pop_clip();
		return;
	}
	let path = parse_shape(&shape.into_mask(), &transform);
	
	dt.fill(&path, &Source::Solid(SolidSource::from_unpremultiplied_argb(style.fill[3], style.fill[0], style.fill[1], style.fill[2])), &DrawOptions::new());
	dt.stroke(&path, 
		&Source::Solid(SolidSource::from_unpremultiplied_argb(style.stroke_color[3], style.stroke_color[0], style.stroke_color[1], style.stroke_color[2])),
		&StrokeStyle { width: style.stroke_width, ..Default::default() }, &DrawOptions::new());
	dt.pop_clip()
}

fn parse_shape(input: &ShapeMask, transform: &Transform) -> Path {
	let mut pb = PathBuilder::new();
	match input {
		ShapeMask::Circle(cir) => {
			pb.arc(cir.radius, cir.radius, cir.radius, 0.0, 2.0 * PI);
			pb.close();
			pb.finish()
		},
		ShapeMask::Rect(rect) => {
			if rect.rounding == Vec2::ZERO {
				pb.rect(0.0, 0.0, rect.width_and_height.x, rect.width_and_height.y);
			}else {
				let magic_number = 4.0 / 3.0 * (2.0_f32.sqrt() - 1.0);
				pb.move_to(rect.width_and_height.x, rect.width_and_height.y - rect.rounding.y);
				pb.cubic_to(rect.width_and_height.x, rect.width_and_height.y + (magic_number - 1.0) * rect.rounding.y,
					rect.width_and_height.x + (magic_number - 1.0) * rect.rounding.x, rect.width_and_height.y,
					rect.width_and_height.x - rect.rounding.x, rect.width_and_height.y);

				pb.line_to(rect.rounding.x, rect.width_and_height.y);
				pb.cubic_to((1.0 - magic_number) * rect.rounding.x, rect.width_and_height.y,
					0.0, rect.width_and_height.y + (magic_number - 1.0) *rect.rounding.y,
					0.0, rect.width_and_height.y - rect.rounding.y);

				pb.line_to(0.0, rect.rounding.y);
				pb.cubic_to(0.0, (1.0 - magic_number) * rect.rounding.y,
					(1.0 - magic_number) * rect.rounding.x, 0.0,
					rect.rounding.x, 0.0);

				pb.line_to(rect.width_and_height.x - rect.rounding.x, 0.0);
				pb.cubic_to(rect.width_and_height.x + (magic_number - 1.0) * rect.rounding.x, 0.0,
					rect.width_and_height.x, (1.0 - magic_number) * rect.rounding.y,
					rect.width_and_height.x, rect.rounding.y);
				pb.line_to(rect.width_and_height.x, rect.width_and_height.y - rect.rounding.y);
				pb.close();
			}
			pb.finish()
		},
		ShapeMask::CubicBezier(cb) => {
			pb.move_to(cb.points[0].x, cb.points[0].y);
			pb.cubic_to(cb.points[1].x, cb.points[1].y, cb.points[2].x, cb.points[2].y, cb.points[3].x, cb.points[3].y);
			if cb.if_close {
				pb.close();
			}
			pb.finish()
		},
		ShapeMask::Polygon(polygon) => {
			for point in polygon.clone().into_iter() {
				pb.line_to(point.x, point.y);
			}
			pb.line_to(polygon[0].x, polygon[0].y);
			pb.close();
			pb.finish()
		},
		ShapeMask::Line(pt) => {
			let angle = pt.angle();
			let points = [Vec2::polar(2.0, angle + PI / 2.0), Vec2::polar(2.0, angle - PI / 2.0), Vec2::polar(2.0, angle - PI / 2.0) + *pt, Vec2::polar(2.0, angle + PI / 2.0) + *pt, Vec2::polar(2.0, angle + PI / 2.0)];
			for point in points {
				pb.line_to(point.x, point.y);
			}
			pb.close();
			pb.finish()
		},
	}.transform(&transform)
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
				format: wgpu::VertexFormat::Float32x2,
			}
		]
	}
}