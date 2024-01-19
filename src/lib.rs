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

#[cfg(feature = "manager")]
mod manager;

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

use clipboard::ClipboardContext;
use crate::event::Touch;
use crate::texture::Image;
use crate::event::OutputEvent;
use crate::integrator::Integrator;
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

#[cfg(feature = "manager")]
use winit::event_loop::ControlFlow;

#[cfg(feature = "vertexs")]
use nablo_shape::prelude::ShapeElement;
#[cfg(feature = "vertexs")]
use nablo_shape::prelude::shape_elements::Text as ShapeText;
#[cfg(feature = "vertexs")]
use nablo_shape::prelude::shape_elements::Style as ShapeStyle;
#[cfg(feature = "vertexs")]
use nablo_shape::prelude::shape_elements::Image as ShapeImage;
#[cfg(feature = "vertexs")]
use std::collections::BTreeMap;
#[cfg(feature = "vertexs")]
use nablo_shape::shape::shape_elements::Vertex;

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

/// a setting to Manager
#[cfg(feature = "manager")]
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
}

/// a trait for your app
#[cfg(feature = "manager")]
pub trait App {
	/// where you add widgets
	fn app(&mut self, ui: &mut Ui);
}

/// your handle to nablo
#[cfg(feature = "manager")]
pub struct Manager<T: App> {
	/// settings to Manager, such as window size.
	pub settings: Settings,
	clipboard: Option<ClipboardContext>,
	image_memory: HashMap<String, Image>,
	/// where you add wigets
	integrator: Integrator,
	/// your app
	pub app: T
}

#[derive(Clone)]
pub(crate) struct MemoryTemp {
	pub response: Response,
	pub access_time: usize
}

#[derive(Default)]
pub(crate) struct Shapes {
	pub raw_shape: Vec<Shape>,
	#[cfg(feature = "vertexs")]
	pub layered_shapes: (Vec<Vertex>, Vec<u32>),
	#[cfg(feature = "vertexs")]
	pub layered_texts: BTreeMap<Layer, Vec<(ShapeText, ShapeStyle)>>,
	#[cfg(feature = "vertexs")]
	pub layered_images: BTreeMap<Layer, Vec<(ShapeImage, ShapeStyle)>>,
	#[cfg(feature = "vertexs")]
	pub current_layer: Layer,
	#[cfg(feature = "vertexs")]
	pub layer_vec: [usize; 6],
}

impl Shapes {
	pub fn append(&mut self, shape: impl Into<Vec<Shape>>) {
		self.raw_shape.append(&mut shape.into())
	}

	#[cfg(feature = "vertexs")]
	pub fn push_shape(&mut self, layer: Layer, shape: (Vec<Vertex>, Vec<u32>)) {
		if layer > self.current_layer {
			self.current_layer = layer
		}
		let (vertexs, indices) = &mut self.layered_shapes;
		self.layer_vec[layer.into_id()] = self.layer_vec[layer.into_id()] + vertexs.len();
		let (mut input_vertexs, mut input_indices) = shape;
		vertexs.append(&mut input_vertexs);
		indices.append(&mut input_indices);
	}

	#[cfg(feature = "vertexs")]
	pub fn push_text(&mut self, layer: Layer, shape: Shape) {
		if let Some(t) = self.layered_texts.get_mut(&layer) {
			if let ShapeElement::Text(text) = shape.shape {
				t.push((text, shape.style))
			}
		}else {
			if let ShapeElement::Text(text) = shape.shape {
				self.layered_texts.insert(layer, vec!((text, shape.style)));
			}
			
		}
	}

	#[cfg(feature = "vertexs")]
	pub fn push_image(&mut self, layer: Layer, shape: Shape) {
		if let Some(t) = self.layered_images.get_mut(&layer) {
			if let ShapeElement::Image(image) = shape.shape {
				t.push((image, shape.style))
			}
		}else {
			if let ShapeElement::Image(image) = shape.shape {
				self.layered_images.insert(layer, vec!((image, shape.style)));
			}
			
		}
	}

	#[cfg(feature = "vertexs")]
	pub fn text_vec(&mut self) -> Vec<(ShapeText, ShapeStyle)> {
		let mut back = vec!();
		for (_, a) in &mut self.layered_texts {
			back.append(a)
		}
		back
	}

	#[cfg(feature = "vertexs")]
	pub fn shape_vec(&mut self) -> (Vec<Vertex>, Vec<u32>) {
		let mut vertexs = vec!();
		let mut indices = vec!();
		let (ver, ind) = &mut self.layered_shapes;
		vertexs.append(ver);
		indices.append(ind);
		(vertexs, indices)
	}

	#[cfg(feature = "vertexs")]
	pub fn vertexs_len(&self) -> u32 {
		let (ver, _) = &self.layered_shapes;
		ver.len() as u32
	}

	#[cfg(not(feature = "vertexs"))]
	pub fn clear(&mut self) {
		self.raw_shape.clear();
	}

	#[cfg(feature = "vertexs")]
	pub fn clear(&mut self) {
		self.raw_shape.clear();
		self.layered_shapes = (vec!(), vec!());
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
	touch: HashMap<usize, (Touch, bool)>,
	input_text: String,
}

#[derive(PartialEq, Clone)]
struct MouseState {
	button: MouseButton,
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
	hover_info: HoverInfo,
	click_info: ClickInfo,
	drag_info: DragInfo,
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
}

#[derive(Clone, Default)]
pub(crate) struct DragInfo {
	drag_start_time: Option<Instant>,
	last_drag_start_position: Option<Vec2>,
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
	/// `nablo` need a id to identify this container
	fn get_id(&self, ui: &mut Ui) -> String;
	/// we must know how large this container is to show it correctly, contains where you should put your container
	fn area(&self, ui: &mut Ui) -> Area;
	/// `nablo` need to know which layer your need to put your widgets
	fn layer(&self, ui: &mut Ui) -> Layer;
	/// handle logic part for your containner before showing widgets. 
	/// the input painter will use as the painter to draw widgets, the painter's offset will be used as container's offect. 
	/// returns if there's need show inner widgets, true for show.
	fn begin(&mut self, ui: &mut Ui, painter: &mut Painter, response: &Response) -> bool;
	/// handle logic part for your containner after showing widgets.
	fn end<R>(&mut self, ui: &mut Ui, painter: &mut Painter, inner_response: &InnerResponse<R>);
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