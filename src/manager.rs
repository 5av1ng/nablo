use winit::event_loop::ControlFlow;
use nablo_shape::prelude::Vec2;
use crate::state::State;
use crate::Key;
use clipboard::ClipboardProvider;
use crate::OutputEvent;
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

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

impl<T: App + 'static> Manager<T> {
	/// run your app
	pub fn run(self) -> Result<()> {
		self.run_app(None)
	}

	/// run your app in given frames
	pub fn run_limited_frames(self, frame: usize) -> Result<()> {
		self.run_app(Some(frame))
	}

	fn handle_event_loop<E>(mut self, frame: Option<usize>, event_loop: EventLoop<E>) -> Result<()> {
		#[cfg(target_os = "android")]
		self.app.android_app(self.android_app.clone());

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

		let mut id = 1;
		cfg_if::cfg_if! {
			if #[cfg(target_arch = "wasm32")] {
				use winit::platform::web::WindowExtWebSys;
				event_loop.set_control_flow(self.settings.control_flow);
				let w_bind;
				if let Some(t) = self.settings.size {
					w_bind = WindowBuilder::new().build(&event_loop).unwrap();
					w_bind.set_min_inner_size(Some(LogicalSize::new(t.x as f64, t.y as f64)));
					self.integrator.event(&NabloEvent::Resized(t));
				}else {
					w_bind = WindowBuilder::new().with_inner_size(LogicalSize::new(640.0,480.0)).build(&event_loop).unwrap();
					self.integrator.event(&NabloEvent::Resized(Vec2::new(640.0,480.0)));
				}
				w_bind.set_title(&self.settings.title);
				w_bind.set_resizable(self.settings.resizeable);
				w_bind.set_ime_allowed(true);
				if self.settings.fullscreen {
					w_bind.set_fullscreen(Some(Fullscreen::Borderless(None)))
				};
				if let Some((color, size)) = &self.settings.icon {
					w_bind.set_window_icon(Some(Icon::from_rgba(color.clone(), size.x as u32, size.y as u32).unwrap()))
				}
				let window = w_bind;

				web_sys::window()
					.and_then(|win| win.document())
					.map(|doc| {
						match doc.get_element_by_id("main") {
							Some(dst) => {
								let _ = dst.append_child(&web_sys::Element::from(window.canvas().unwrap()));
							}
							None => {
								let canvas = window.canvas().unwrap();
								doc.body().map(|body| body.append_child(&web_sys::Element::from(canvas)));
							}
						};
					}).expect("cant run");

				let mut state = State::new(&window, Vec2::new(window.inner_size().width as f32, window.inner_size().height as f32));
				
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
										if self.settings.soft_rendering {
											let output = self.integrator.frame(vec!(), |ui| self.app.app(ui));
											for event in &output.output_events {
												self.handle_event(event.clone(), &mut state)
											}
											match state.soft_render(output) {
												Ok(_) => {}
												Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
												Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
												Err(e) => eprintln!("{:?}", e),
											};
										}else {
											let output = self.integrator.frame_vertexs(vec!(), |ui| self.app.app(ui));
											for event in &output.output_events {
												self.handle_event(event.clone(), &mut state)
											}
											match state.render(output) {
												Ok(_) => {}
												Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
												Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
												Err(e) => eprintln!("{:?}", e),
											};
										}
										
										if let Some(inner) = frame {
											if id > inner {
												elwt.exit()
											}
										}
										id = id + 1;
										window.request_redraw();
									},
									WindowEvent::CloseRequested => { elwt.exit() },
									WindowEvent::Resized(physical_size) => {
										state.resize(Vec2::new(physical_size.width as f32, physical_size.height as f32))
									},
									_ => {}
								}
							}
						},
						_ => {}
					}
				}).unwrap();

				Ok(())
			}else {
				let mut window: Option<winit::window::Window> = None;
				let mut state: Option<State> = None;
				event_loop.run(move |winit_event, elwt, control_flow| {
					*control_flow = self.settings.control_flow;
					match winit_event {
						Event::Resumed => {
							let w_bind;
							if let Some(t) = self.settings.size {
								w_bind = WindowBuilder::new().build(&elwt).unwrap();
								w_bind.set_min_inner_size(Some(LogicalSize::new(t.x as f64, t.y as f64)));
								self.integrator.event(&NabloEvent::Resized(t));
							}else {
								w_bind = WindowBuilder::new().with_inner_size(LogicalSize::new(640.0,480.0)).build(&elwt).unwrap();
								self.integrator.event(&NabloEvent::Resized(Vec2::new(640.0,480.0)));
							}
							w_bind.set_title(&self.settings.title);
							w_bind.set_resizable(self.settings.resizeable);
							w_bind.set_ime_allowed(true);
							if self.settings.fullscreen {
								w_bind.set_fullscreen(Some(Fullscreen::Borderless(None)))
							};
							if let Some((color, size)) = &self.settings.icon {
								w_bind.set_window_icon(Some(Icon::from_rgba(color.clone(), size.x as u32, size.y as u32).unwrap()))
							}
							window = Some(w_bind);
							state = Some(State::new(window.as_ref().unwrap(), Vec2::new(window.as_ref().unwrap().inner_size().width as f32, window.as_ref().unwrap().inner_size().height as f32)));
						},
						Event::Suspended => {
							window = None;
							state = None;
						},
						Event::WindowEvent {
							event,
							window_id: _,
						} => {
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
								WindowEvent::CloseRequested => {*control_flow = ControlFlow::Exit},
								WindowEvent::Resized(physical_size) => {
									if let Some(state) = &mut state {
										state.resize(Vec2::new(physical_size.width as f32, physical_size.height as f32))
									}
								},
								_ => {}
							}
							self.integrator.event(&event.into());
						},
						Event::RedrawRequested(_) => {
							if let Some(state) = &mut state {
								if self.settings.soft_rendering {
									let output = self.integrator.frame(vec!(), |ui| self.app.app(ui));
									for event in &output.output_events {
										self.handle_event(event.clone(), state)
									}
									match state.soft_render(output) {
										Ok(_) => {}
										Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
										Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
										Err(e) => eprintln!("{:?}", e),
									};
								}else {
									let output = self.integrator.frame_vertexs(vec!(), |ui| self.app.app(ui));
									for event in &output.output_events {
										self.handle_event(event.clone(), state)
									}
									match state.render(output) {
										Ok(_) => {}
										Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
										Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
										Err(e) => eprintln!("{:?}", e),
									};
								}
								
								if let Some(inner) = frame {
									if id > inner {
										*control_flow = ControlFlow::Exit
									}
								}
								id = id + 1;
								if let Some(t) = &window {
									t.request_redraw();
								}
							}
						},
						Event::RedrawEventsCleared => {
							if let Some(t) = &window {
								t.request_redraw();
							}
						},
						_ => {}
					}
				});
			}
		}
		
	}

	#[cfg(not(target_os = "android"))]
	fn run_app(self, frame: Option<usize>) -> Result<()> {
		let event_loop = EventLoop::new();
		#[cfg(target_arch = "wasm32")]
		let event_loop = event_loop.unwrap();
		self.handle_event_loop(frame, event_loop)
	}

	#[cfg(target_os = "android")]
	fn run_app(self, frame: Option<usize>) -> Result<()> {
		use winit::platform::android::EventLoopBuilderExtAndroid;

		let event_loop = winit::event_loop::EventLoopBuilder::new().with_android_app(self.android_app.clone()).build();
		#[cfg(target_arch = "wasm32")]
		let event_loop = event_loop.unwrap();
		self.handle_event_loop(frame, event_loop)
	}

	#[cfg(not(target_os = "android"))]
	/// create a manager for your app
	pub fn new(app: T) -> Self {
		Self {
			settings: Settings::default(),
			integrator: Integrator::default(),
			app,
			clipboard: None
		}
	}

	#[cfg(not(target_os = "android"))]
	/// create a manager for your app with given settings.
	pub fn new_with_settings(app: T, settings: Settings) -> Self {
		Self {
			settings,
			integrator: Integrator::default(),
			app,
			clipboard: None
		}
	}

	#[cfg(target_os = "android")]
	/// create a manager for your app
	pub fn new(app: T, android_app: AndroidApp) -> Self {
		Self {
			settings: Settings::default(),
			integrator: Integrator::default(),
			app,
			clipboard: None,
			android_app,
		}
	}

	#[cfg(target_os = "android")]
	/// create a manager for your app with given settings.
	pub fn new_with_settings(app: T, settings: Settings, android_app: AndroidApp) -> Self {
		Self {
			settings,
			integrator: Integrator::default(),
			app,
			clipboard: None,
			android_app,
		}
	}

	#[cfg(not(target_os = "android"))]
	fn handle_event(&mut self, event: OutputEvent, state: &mut State) {
		match event {
			OutputEvent::TextureCreate(texture) => {
				state.insert_texture(texture.id.clone(), texture);
			},
			OutputEvent::TextureChange(texture) => {
				state.insert_texture(texture.id.clone(), texture);
			},
			OutputEvent::TextureDelete(id) => {
				state.remove_texture(&id);
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

	#[cfg(target_os = "android")]
	fn handle_event(&mut self, event: OutputEvent, state: &mut State) {
		match event {
			OutputEvent::TextureCreate(texture) => {
				state.insert_texture(texture.id.clone(), texture);
			},
			OutputEvent::TextureChange(texture) => {
				state.insert_texture(texture.id.clone(), texture);
			},
			OutputEvent::TextureDelete(id) => {
				state.remove_texture(&id);
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
			OutputEvent::RequireSoftKeyboard(inner) => {
				if inner {
					self.android_app.show_soft_input(true);
				}else {
					self.android_app.hide_soft_input(true);
				}
			},
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
			#[cfg(target_os = "android")]
			soft_rendering: true,
			#[cfg(not(target_os = "android"))]
			soft_rendering: false,
		}
	}
}