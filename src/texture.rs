//! texture the host will handle.

#[cfg(any(feature = "manager", feature = "baseview_manager"))]
use crate::state::WTexture;
use nablo_shape::prelude::Vec2;

/// a image to be added.
#[derive(Clone, Debug, PartialEq)]
pub struct Image {
	pub rgba: Vec<u8>,
	pub id: String,
	pub size: Vec2
}

#[cfg(any(feature = "manager", feature = "baseview_manager"))]
pub(crate) fn create_texture(size: Vec2, device: &wgpu::Device, queue: &wgpu::Queue) -> WTexture {
	let shapes: Vec<u8> = (0..(size.x * size.y * 4.0) as usize).into_iter().map(|_| 0u8).collect();
	create_texture_with_data(size, device, queue, shapes)
}

#[cfg(any(feature = "manager", feature = "baseview_manager"))]
pub(crate) fn create_texture_with_data(size: Vec2, device: &wgpu::Device, queue: &wgpu::Queue, data: Vec<u8>) -> WTexture {
	let texture_size = wgpu::Extent3d {
		width: size.x as u32,
		height: size.y as u32,
		depth_or_array_layers: 1,
	};

	let diffuse_texture = device.create_texture(
		&wgpu::TextureDescriptor {
			size: texture_size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba8UnormSrgb,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
			label: Some("main_texture"),
			view_formats: &[],
		}
	);

	let slice = data.as_slice();

	queue.write_texture(wgpu::ImageCopyTexture {
		texture: &diffuse_texture,
		mip_level: 0,
		origin: wgpu::Origin3d::ZERO,
		aspect: wgpu::TextureAspect::All,
	}, &slice, wgpu::ImageDataLayout {
		offset: 0,
		bytes_per_row: Some(size.x as u32 * 4),
		rows_per_image: Some(size.y as u32),
	}, texture_size);
	let diffuse_texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
	let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
		address_mode_u: wgpu::AddressMode::MirrorRepeat,
		address_mode_v: wgpu::AddressMode::MirrorRepeat,
		address_mode_w: wgpu::AddressMode::MirrorRepeat,
		mag_filter: wgpu::FilterMode::Linear,
		min_filter: wgpu::FilterMode::Nearest,
		mipmap_filter: wgpu::FilterMode::Nearest,
		..Default::default()
	});
	let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		entries: &[wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
					multisampled: false,
					view_dimension: wgpu::TextureViewDimension::D2,
					sample_type: wgpu::TextureSampleType::Float { filterable: true },
				},
			count: None,
			},
			wgpu::BindGroupLayoutEntry {
				binding: 1,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
				count: None,
			}
		],
		label: Some("texture_bind_group_layout"),
	});

	let diffuse_bind_group = device.create_bind_group(
		&wgpu::BindGroupDescriptor {
			layout: &texture_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
				}
			],
			label: Some("diffuse_bind_group"),
		}
	);
	WTexture {
		texture: diffuse_texture,
		layout: texture_bind_group_layout,
		bind_group: diffuse_bind_group
	}
}

#[cfg(any(feature = "manager", feature = "baseview_manager"))]
pub(crate) fn write_texture_with_data(size: Vec2, texture: &wgpu::Texture, queue: &wgpu::Queue, data: &[u8]) {
	let texture_size = wgpu::Extent3d {
		width: size.x as u32,
		height: size.y as u32,
		depth_or_array_layers: 1,
	};

	queue.write_texture(wgpu::ImageCopyTexture {
		texture: &texture,
		mip_level: 0,
		origin: wgpu::Origin3d::ZERO,
		aspect: wgpu::TextureAspect::All,
	}, &data, wgpu::ImageDataLayout {
		offset: 0,
		bytes_per_row: Some(size.x as u32 * 4),
		rows_per_image: Some(size.y as u32),
	}, texture_size);
}
