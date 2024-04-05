/*! `nablo`: a gui library, not too slow, easy to use.
 * 
 * `nablo` is a immediate mode ui inspired by [egui](https://crates.io/crates/egui).
 * to get more intuitive examples, check the online example (TODO)
 * 
 * # A Simple Example
 * ```no_run
 * // lets just start with a simple counter demo
 * 
 * use nablo::prelude::*;
 * 
 * // we should have a struct to save our important datas. though nablo has own memeory, but it isnt designed for long-time storge.
 * #[derive(Default)]
 * struct Counter {
 *     // feel free to add stuff here
 *     counter: i32,
 * }
 * 
 * // a app trait for your application
 * impl App for Counter {
 *      fn app(&mut self, ui: &mut Ui) {
 *          // there would be where you add widgets. check build-in widgets in widgets module.
 *          if ui.add(Button::new("+")).is_clicked() {
 *              self.counter += 1
 *          }
 *          if ui.add(Button::new("-")).is_clicked() {
 *              self.counter -= 1
 *          }
 *          ui.add(Label::new(format!("counter: {}", self.counter)));
 *      }
 * } 
 * 
 * fn main() {
 *      // nablo build-in window manager
 *      Manager::new(Counter::default()).run();
 * }
 * ```
 */

#[cfg(all(feature = "manager", feature = "baseview_manager"))]
compile_error!("feature \"manager\" and feature \"baseview_manager\" cannot be enabled at the same time");

cfg_if::cfg_if! {
	if #[cfg(feature = "manager")] {
		mod manager;
		mod state;
		use clipboard::ClipboardContext;
		use crate::integrator::Integrator;
		use winit::event_loop::ControlFlow;
	}else if #[cfg(feature = "baseview_manager")] {
		mod baseview_manager;
		mod state;
		use clipboard::ClipboardContext;
		use crate::integrator::Integrator;
	}
}

cfg_if::cfg_if! {
	if #[cfg(feature = "vertexs")] {
		use crate::integrator::ParsedShape;
	}
}

mod ui;
mod response;
pub mod texture;
pub mod event;
pub mod widgets;
pub mod container;
pub mod prelude;
pub mod integrator;
#[cfg(feature = "presets")]
pub mod presets;


use crate::event::Touch;
use crate::event::OutputEvent;
use std::ops::Sub;
use time::Duration;
use nablo_shape::shape::shape_elements::Layer;
use crate::widgets::Style;
use nablo_shape::shape::shape_elements::Style as PaintStyle;
use nablo_shape::shape::Painter;
use crate::event::MouseButton;
use crate::event::Key;
use crate::event::Event;
use nablo_shape::shape::Shape;
use std::collections::HashMap;
use nablo_shape::math::Area;
use nablo_shape::math::Vec2;
use time::OffsetDateTime;


/// will replace when typing in a input setted `is_password = true`
pub const PASSWORD: char = '●';

#[derive(Clone, serde::Serialize, serde::Deserialize, Copy, Debug)]
pub(crate) struct Instant {
	offset: OffsetDateTime
}

impl Instant {
	pub fn now() -> Self {
		Self {
			offset: OffsetDateTime::now_utc()
		}
	}

	pub fn elapsed(&self) -> Duration {
		Self::now().offset - self.offset
	}
}

impl Sub for Instant {
	type Output = Duration;

	fn sub(self, rhs: Self) -> Self::Output {
		self.offset - rhs.offset
	}
}

cfg_if::cfg_if!{
	if #[cfg(feature = "manager")] {
		/// a setting to Manager
		pub struct Settings {
			/// how much clicks should we save?
			pub max_clicks: usize,
			/// how large is our window? if window is resizeable, this would be min size
			pub size: Option<Vec2>,
			pub title: String,
			pub resizeable: bool,
			pub fullscreen: bool,
			pub icon: Option<(Vec<u8>,Vec2)>,
			pub control_flow: ControlFlow,
			pub soft_rendering: bool,
		}

		/// a trait for your app
		pub trait App {
			/// where you add widgets
			fn app(&mut self, ui: &mut Ui);
			#[cfg(target_os = "android")]
			/// you may want handle android main when in android platform, this will run before actually running window manage process.
			fn android_app(&mut self, app: prelude::AndroidApp);
		}

		/// your handle to nablo
		pub struct Manager<T: App> {
			/// settings to Manager, such as window size.
			pub settings: Settings,
			clipboard: Option<ClipboardContext>,
			/// where you add wigets
			integrator: Integrator,
			/// your app
			pub app: T,
			#[cfg(target_os = "android")]
			pub android_app: winit::platform::android::activity::AndroidApp,
		}
	}else if #[cfg(feature = "baseview_manager")] {
		use state::State;
		/// a setting to Manager
		#[derive(Clone)]
		pub struct Settings {
			/// how much clicks should we save?
			pub max_clicks: usize,
			/// how large is our window? if window is resizeable, this would be min size
			pub size: Vec2,
			pub title: String,
		}

		/// a trait for your app
		pub trait App {
			/// where you add widgets
			fn app(&mut self, ui: &mut Ui);
		}

		/// a builder to build [`Manger`]
		pub struct ManagerBuilder<T: App> {
			pub settings: Settings,
			pub app: T,
		}

		impl<T> App for T where 
			T: Fn(&mut Ui)
		{
			fn app(&mut self, ui: &mut Ui) {
				self(ui);
			}
		}

		/// your handle to nablo
		pub struct Manager<T: App> {
			/// settings to Manager, such as window size.
			pub settings: Settings,
			clipboard: Option<ClipboardContext>,
			/// where you add wigets
			integrator: Integrator,
			/// your app
			pub app: T,
			state: State
		}
	}
}

#[derive(Clone)]
pub(crate) struct MemoryTemp {
	pub response: Response,
	pub update_area: Area,
	pub access_time: usize
}

#[derive(Default)]
pub(crate) struct Shapes {
	pub raw_shape: Vec<Shape>,
	#[cfg(feature = "vertexs")]
	pub parsed_shapes: Vec<ParsedShape>
}

impl Shapes {
	pub fn append(&mut self, shape: impl Into<Vec<Shape>>) {
		self.raw_shape.append(&mut shape.into())
	}

	#[cfg(not(feature = "vertexs"))]
	pub fn clear(&mut self) {
		self.raw_shape.clear();
	}

	#[cfg(feature = "vertexs")]
	pub fn clear(&mut self) {
		self.raw_shape.clear();
		self.parsed_shapes.clear();
	}
}

/// what you use for adding your widgets
pub struct Ui {
	memory: HashMap<String, MemoryTemp>,
	memory_clip: Vec<String>,
	memory_clip_total: Vec<String>,
	shape: Shapes,
	last_frame: Instant,
	available_position: Vec2,
	input_state: InputState,
	events: Vec<Event>,
	window: Area,
	style: Style,
	available_id: (String, usize),
	language: String,
	paint_style: PaintStyle,
	layout: Layout,
	output_events: Vec<OutputEvent>,
	texture_id: Vec<String>,
	offset: Vec2,
	parent_area: Option<Area>,
	start_position: Vec2,
	window_crossed: Area,
}

#[derive(Default, Clone)]
/// a struct used for layout during ui, for advanced layout options, you can use [`crate::container::Card`] as a replace.
///
/// very basic for now.
pub struct Layout {
	/// if this value is false, [`Ui`] will put the content left to right or top to bottom, otherwise will put the content right to left or bottom to top 
	is_inverse: bool,
	/// if true, this will put the content horizentally.
	is_horizental: bool,
	// /// if true, [`Ui`] will put the content in central position.
	// is_centered: bool
	// /// this will affect the position [`Ui`] put.
	// align: Align
}

// #[derive(Default)]
// /// left/center/right or top/center/bottom alignment
// pub enum Align {
// 	#[default] Left,
// 	Right,
// 	Center
// }

/// where you get current key or mouse position
///
/// can be used by calling [`Ui::input()`]
#[derive(Default, Clone)]
pub struct InputState {
	key: Vec<Key>,
	key_repeat: Vec<Key>,
	released_key: Vec<Key>,
	cursor_position: Option<Vec2>,
	pressed_mouse: Vec<MouseState>,
	pressing_mouse: Vec<(MouseButton, bool)>,
	released_mouse: Vec<(MouseButton, bool)>,
	click_time: HashMap<MouseButton, Instant>,
	is_ime_on: bool,
	current_scroll: Vec2,
	touch: HashMap<usize, TouchState>,
	input_text: String,
}

#[derive(PartialEq, Clone)]
struct MouseState {
	button: MouseButton,
	is_drag_used: bool,
	is_click_used: bool
}

#[derive(PartialEq, Clone)]
#[derive(Default)]
struct TouchState {
	touch: Touch,
	is_drag_used: bool,
	is_click_used: bool
}

impl Into<MouseState> for MouseButton {
	fn into(self) -> MouseState {
		MouseState {
			button: self,
			is_drag_used: false,
			is_click_used: false
		}
	}
}

/// anything implementing Widget can be added by using [`Ui::add_with_id`]
pub trait Widget {
	/// tell `nablo` how to draw your widget
	fn draw(&mut self, ui: &mut Ui, response: &Response, painter: &mut Painter);
	/// tell `nablo` where your widgets is, the area represents where `nablo` hope your put your widget at.
	fn ui(&mut self, ui: &mut Ui, area: Option<Area>) -> Response;
}

#[derive(Clone, Default)]
/// the result of adding a widget to a [`Ui`], represents a widget.
pub struct Response {
	/// where we are and how large area we take 
	pub area: Area,
	/// id of this widget
	pub id: String,
	/// saves datas such as how long since this widget add.
	metadata: Metadata,
}

#[derive(Clone)]
/// saves datas such as how long since a widget add.
pub(crate) struct Metadata {
	layer: Layer,
	create_time: Instant,
	pointer_position: Option<Vec2>,
	hover_info: Option<HoverInfo>,
	click_info: Option<ClickInfo>,
	drag_info: Option<DragInfo>,
	other_info: String,
}

#[derive(Clone, Default)]
pub(crate) struct HoverInfo {
	last_hover_time: Option<Instant>,
	last_lost_hover_time: Option<Instant>,
	is_hovering: bool,
	is_insert: bool
}

#[derive(Clone, Default)]
pub(crate) struct ClickInfo {
	press_time: Vec<Instant>,
	release_time: Vec<Instant>,
	last_click_position: Option<Vec2>,
	pressed_mouse: Vec<MouseButton>,
	released_mouse: Vec<MouseButton>,
	pressed_touch: Vec<Touch>,
	released_touch: Vec<Touch>,
	is_pressed: bool,
	is_released: bool,
}

#[derive(Clone, Default)]
pub(crate) struct DragInfo {
	drag_start_time: Option<Instant>,
	last_drag_start_position: Option<Vec2>,
	last_drag_position: Option<Vec2>,
	drag_delta: Vec2,
	is_draging: bool
}

/// a skin to [`Response`] when we want to get return types in ui functions
pub struct InnerResponse<R> {
	/// [`Option::None`] for not show inner.
	pub return_value: Option<R>,
	pub inner_responses: Vec<Response>,
	/// containners response
	pub response: Response,
}

impl<R> From<(Response,Vec<Response> , R)> for InnerResponse<R> {
	fn from(input: (Response, Vec<Response>, R)) -> InnerResponse<R> {
		let (response, inner_responses, return_value) = input;
		InnerResponse {
			response,
			inner_responses,
			return_value: Some(return_value),
		}
	}
}

/// any thing implied this trait would able to be a container.
pub trait Container {
	/// `nablo` need a id to identify this container, Note: this is not actual id of current Container, use [`Ui::container_id()`] to get actual id
	fn get_id(&self, ui: &mut Ui) -> String;
	/// as name says, will not affect widgets put on it
	fn is_clickable(&self, ui: &mut Ui) -> bool;
	/// as name says, will not affect widgets put on it
	fn is_dragable(&self, ui: &mut Ui) -> bool;
	/// we must know how large this container is to show it correctly, contains where you should put your container
	fn area(&self, ui: &mut Ui) -> Area;
	/// `nablo` need to know which layer your need to put your widgets
	fn layer(&self, ui: &mut Ui) -> Layer;
	/// handle logic part for your containner before showing widgets. 
	/// the input painter will use as the painter to draw widgets, the painter's offset will be used as container's offect. 
	/// returns if there's need show inner widgets, true for show.
	fn begin(&mut self, ui: &mut Ui, painter: &mut Painter, response: &Response, id: &String) -> bool;
	/// handle logic part for your containner after showing widgets.
	fn end<R>(&mut self, ui: &mut Ui, painter: &mut Painter, inner_response: &InnerResponse<R>, id: &String);
}

pub(crate) fn parse_json<T: for<'a> serde::Deserialize<'a> + Default>(input: &String) -> T  {
	match serde_json::from_str(input) {
		Ok(t) => return t,
		Err(_) => {
			return T::default()
		}
	};
}

pub(crate) fn to_json<T: serde::Serialize>(input: &T) -> String {
	match serde_json::to_string_pretty(input) {
		Ok(t) => return t,
		Err(_) => {
			return String::new()
		}
	};
}