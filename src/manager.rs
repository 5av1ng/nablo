use winit::event_loop::ControlFlow;
use nablo_shape::prelude::Vec2;
use crate::state::State;
use crate::Key;
use clipboard::ClipboardProvider;
use fontdue::Font;
use crate::OutputEvent;
use std::collections::HashMap;
use std::result::Result::Ok;
use crate::Integrator;
use crate::Manager;
use crate::App;
use crate::Settings;
use winit::event::Event;
use crate::event::Event as NabloEvent;
use winit::window::Icon;
use winit::window::Fullscreen;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use winit::event::WindowEvent;
use anyhow::*;

impl<T: App> Manager<T> {
	/// run your app
	pub fn run(&mut self) -> Result<()> {
		let event_loop = EventLoop::new()?;
		let window;
		if let Some(t) = self.settings.size {
			window = WindowBuilder::new().build(&event_loop)?;
			window.set_min_inner_size(Some(LogicalSize::new(t.x as f64, t.y as f64)));
		}else {
			window = WindowBuilder::new().with_inner_size(LogicalSize::new(640.0,480.0)).build(&event_loop)?;
		}
		window.set_title(&self.settings.title);
		window.set_resizable(self.settings.resizeable);
		window.set_ime_allowed(true);
		// TODO: make this changable
		self.clipboard = match ClipboardProvider::new() {
			Ok(t) => Some(t),
			Err(_) => {
				#[cfg(feature = "info")]
				println!("initlizing clipboard failed, will not using clipboard...");
				#[cfg(feature = "log")]
				log::warn!("initlizing clipboard failed, will not using clipboard...");
				None
			}
		};
		fn font(input: &[u8]) -> Result<Font> {
			match fontdue::Font::from_bytes(input, fontdue::FontSettings::default()) {
				Ok(t) => Ok(t),
				Err(e) => return Err(anyhow!("{}", e)),
			}
		}
		let font_1 = font(include_bytes!("../font_normal.ttf") as &[u8])?;
		let font_2 = font(include_bytes!("../font_bold.ttf") as &[u8])?;
		let font_3 = font(include_bytes!("../font_italic.ttf") as &[u8])?;
		let font_4 = font(include_bytes!("../font_bold_italic.ttf") as &[u8])?;
		let fonts = [font_1, font_2, font_3, font_4];
		if self.settings.fullscreen {
			window.set_fullscreen(Some(Fullscreen::Borderless(None)))
		};
		if let Some((color, size)) = &self.settings.icon {
			window.set_window_icon(Some(Icon::from_rgba(color.clone(), size.x as u32, size.y as u32)?))
		}
		event_loop.set_control_flow(self.settings.control_flow);

		#[cfg(target_arch = "wasm32")]
		{
			use winit::platform::web::WindowExtWebSys;

			web_sys::window()
				.and_then(|win| win.document())
				.map(|doc| {
					match doc.get_element_by_id("nablo") {
						Some(dst) => {
							let _ = dst.append_child(&web_sys::Element::from(window.canvas().expect("cant sppend canvas")));
						}
						None => {
							let canvas = window.canvas().expect("cant add canvas");
							canvas.set_width(640);
							canvas.set_height(480);
							doc.body().map(|body| body.append_child(&web_sys::Element::from(canvas)));
						}
					};
				}).expect("cant run");
		}
		let mut state = State::new(&window);

		event_loop.run(move |winit_event, elwt| {
			match winit_event {
				Event::WindowEvent {
					event,
					window_id,
				} => {
					if window_id == window.id() {
						self.integrator.event(&event.clone().into());
						if let Some(clipboard) = &mut self.clipboard {
							let input =  self.integrator.ui.input();
							if (input.is_key_pressing(Key::ControlLeft) && input.is_key_pressing(Key::V)) | 
							(input.is_key_released(Key::ControlLeft) && input.is_key_released(Key::V)) |
							(input.is_key_pressing(Key::ControlLeft) && input.is_key_released(Key::V)) {
								let data = clipboard.get_contents();
								if let Ok(data) = data {
									self.integrator.event(&NabloEvent::TextInput(data));
								}else if let Err(e) = data {
									#[cfg(feature = "info")]
									println!("get clipboard info failed, info: {}", e);
									#[cfg(feature = "log")]
									log::error!("get clipboard info failed, info: {}", e);
								};
							}
						} 
						match event {
							WindowEvent::RedrawRequested => {
								let output = self.integrator.frame(vec!(), |ui| self.app.app(ui));
								match state.render(&output, &fonts, &self.image_memory) {
									Ok(_) => {}
									Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
									Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
									Err(e) => eprintln!("{:?}", e),
								};
								for event in output.output_events {
									self.handle_event(event)
								}
								window.request_redraw();
							},
							WindowEvent::CloseRequested => {elwt.exit()},
							WindowEvent::Resized(physical_size) => {
								state.resize(Vec2::new(physical_size.width as f32, physical_size.height as f32))
							},
							_ => {}
						}
					}
				},
				_ => {}
			}
		})?;
		Ok(())
	}

	/// create a manager for your app
	pub fn new(app: T) -> Self {
		Self {
			settings: Settings::default(),
			integrator: Integrator::default(),
			image_memory: HashMap::new(),
			app,
			clipboard: None
		}
	}

	/// create a manager for your app with given settings.
	pub fn new_with_settings(app: T, settings: Settings) -> Self {
		Self {
			settings,
			integrator: Integrator::default(),
			image_memory: HashMap::new(),
			app,
			clipboard: None
		}
	}

	fn handle_event(&mut self, event: OutputEvent) {
		match event {
			OutputEvent::TextureCreate(texture) => {
				self.image_memory.insert(texture.id.clone(), texture);
			},
			OutputEvent::TextureChange(texture) => {
				self.image_memory.insert(texture.id.clone(), texture);
			},
			OutputEvent::TextureDelete(id) => {
				self.image_memory.remove(&id);
			},
			OutputEvent::ClipboardCopy(text) => {
				if let Some(clipboard) = &mut self.clipboard {
					let data = clipboard.set_contents(text);
					if let Err(e) = data {
						#[cfg(feature = "info")]
						println!("get clipboard info failed, info: {}", e);
						#[cfg(feature = "log")]
						log::error!("get clipboard info failed, info: {}", e);
					};
				}
			},
			OutputEvent::RequireSoftKeyboard(_) => {},
		}
	}
}

impl Default for Settings {
	fn default() -> Self {
		Self {
			max_clicks: 5,
			size: Some(Vec2::new(640.0,480.0)),
			title: String::from("nablo"),
			resizeable: true,
			fullscreen: false,
			icon: None,
			control_flow: ControlFlow::Poll,
		}
	}
}