use crate::Integrator;
use crate::state::State;
use crate::Ui;
use crate::ManagerBuilder;
use baseview::WindowHandle;
use raw_window_handle::HasRawWindowHandle;
use baseview::Size;
use baseview::WindowScalePolicy;
use baseview::WindowOpenOptions;
use crate::Event;
use baseview::Event as BaseviewEvent;
use baseview::EventStatus;
use baseview::WindowHandler;
use baseview::Window;
use nablo_shape::prelude::Vec2;
use clipboard::ClipboardProvider;
use crate::OutputEvent;
use crate::Manager;
use crate::App;
use crate::Settings;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	position: [f32; 3],
	tex_coords: [f32; 2],
}

impl<T: App + Send + 'static> WindowHandler for Manager<T> {
	fn on_frame(&mut self, _: &mut Window) {
		let output = self.integrator.frame_vertexs(vec!(), |ui| self.app.app(ui));
		for event in &output.output_events {
			self.handle_event(event.clone())
		}
		match self.state.render(output) {
			Ok(_) => {}
			Err(wgpu::SurfaceError::Lost) => self.state.resize(self.state.size),
			Err(e) => eprintln!("{:?}", e),
		};
		
	}

	fn on_event(&mut self, _: &mut Window<'_>, event: BaseviewEvent) -> EventStatus {
		let event: Event = event.into();
		if let Event::Resized(inner) = event {
			self.state.resize(inner)
		}
		self.integrator.event(&event);
		EventStatus::Captured
	}
}

impl<T: App + Send + 'static> ManagerBuilder<T> {
	/// see more in baseview documentationg
	pub fn open_blocking(self){
		Window::open_blocking(WindowOpenOptions {
			title: self.settings.title.clone(),
			size: Size { width: self.settings.size.x as f64, height: self.settings.size.y as f64 },
			scale: WindowScalePolicy::SystemScaleFactor,
			gl_config: None
		}, |window| Manager::new(window, self));
	}

	/// see more in baseview documentationg
	pub fn open_parented<P>(parent: &P, settings: Settings, app: T) -> WindowHandle where 
		P: HasRawWindowHandle
	{
		Window::open_parented(parent, WindowOpenOptions {
			title: settings.title.clone(),
			size: Size { width: settings.size.x as f64, height: settings.size.y as f64 },
			scale: WindowScalePolicy::SystemScaleFactor,
			gl_config: None
		}, |window| Manager::new(window, Self::new(settings, app)))
	}

	/// create a new manager builder
	pub fn new(settings: Settings, app: T) -> Self {
		Self {
			settings,
			app
		}
	}
}

impl<T> ManagerBuilder<T> where 
	T: Fn(&mut Ui) + Send + Sync
{
	/// create a new manager builder with closure
	pub fn new_closure(settings: Settings, app: T) -> Self {
		Self {
			settings,
			app: app
		}
	}
}

impl<T: App + Send + 'static> Manager<T> {
	/// create a manager for your app
	fn new(window: &Window, builder: ManagerBuilder<T>) -> Self {
		let clipboard = match ClipboardProvider::new() {
			Ok(t) => Some(t),
			Err(_) => {
				#[cfg(feature = "info")]
				println!("initlizing clipboard failed, will not using clipboard...");
				#[cfg(feature = "log")]
				log::warn!("initlizing clipboard failed, will not using clipboard...");
				None
			}
		};
		let state = State::new(window, builder.settings.size);
		let mut integrator = Integrator::default();
		integrator.event(&Event::Resized(builder.settings.size));
		Self {
			settings: Settings::default(),
			integrator,
			app: builder.app,
			clipboard,
			state,
		}
	}

	fn handle_event(&mut self, event: OutputEvent) {
		match event {
			OutputEvent::TextureCreate(texture) => {
				self.state.insert_texture(texture.id.clone(), texture);
			},
			OutputEvent::TextureChange(texture) => {
				self.state.insert_texture(texture.id.clone(), texture);
			},
			OutputEvent::TextureDelete(id) => {
				self.state.remove_texture(&id);
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
			size: Vec2::new(640.0,480.0),
			title: String::from("nablo"),
		}
	}
}