//! widgets provided by `nablo`

use crate::prelude::TooltipProvider;
use crate::prelude::message_provider::Message;
use crate::prelude::Collapsing;
use nablo_shape::prelude::shape_elements::TextStyle;
use crate::Response;
use crate::Widget;
use nablo_shape::prelude::shape_elements::Color;
use nablo_shape::math::Area;
use nablo_shape::math::Vec2;
use nablo_shape::shape::Painter;
use crate::Ui;
use nablo_shape::shape::shape_elements::EM;

mod button;
mod canvas;
mod label;
mod selectable_value;
mod single_input;
mod slider;
mod dragable_value;
mod divide_line;
mod progress_bar;

/// a general style used by all wigets
#[derive(Clone)]
pub struct Style {
	pub background_color: Color,
	pub card_color: Color,
	pub seprator_color: Color,
	pub slider_peek_color: Color,
	pub slider_unreached_color: Color,
	pub slider_reached_color: Color,
	pub primary_color: Color,
	pub text_color: Color,
	pub error_color: Color,
	pub info_color: Color,
	pub warning_color: Color,
	pub success_color: Color,
	pub space: f32,
	pub brighten_factor: f32,
}

impl Default for Style {
	fn default() -> Self { 
		Self {
			background_color: [18,18,18,255].into(),
			card_color: [30, 30, 30, 255].into(),
			seprator_color: [51,51,51, 255].into(),
			slider_peek_color: [118,118,118, 255].into(),
			slider_reached_color: [97,97,97, 255].into(),
			slider_unreached_color: [200, 200, 200, 255].into(),
			primary_color: [98,0,234, 255].into(),
			text_color: [230,230,230,255].into(),
			error_color: [229, 115, 115,255].into(),
			info_color: [100, 181, 246,255].into(),
			warning_color: [255, 235, 59,255].into(),
			success_color: [111,249,6 ,255].into(),
			space: EM,
			brighten_factor: 0.1,
		}
	}
}

/// nablo build-in status, should be enough to use. i think...
#[derive(Default, Clone, serde::Deserialize, serde::Serialize, Debug, PartialEq)]
#[non_exhaustive]
pub enum Status {
	Error,
	Info,
	Warning,
	Success,
	#[default] Default
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize, Debug, PartialEq)]
/// nablo build-in button's style
pub enum ButtonStyle {
	#[default] Normal,
	Stroked,
	Lined
}

/// a text used by all build-in wigets, contains some basic style settings
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Text {
	pub text: String,
	pub status: Status,
	pub color: Option<Color>,
	pub size: Vec2,
	pub width: Option<f32>,
	pub height: Option<f32>,
	pub underline: bool,
	pub style: TextStyle
}

impl Default for Text {
	fn default() -> Self {
		Self {
			text: String::new(),
			status: Status::default(),
			color: None,
			size: Vec2::NOT_TO_SCALE,
			width: None,
			height: None,
			underline: false,
			style: TextStyle::default()
		}
	}
}

impl Text {
	pub fn is_empty(&self) -> bool {
		self.text.is_empty()
	}
}

/// a trait that allows you change text styles.
pub trait TextSetting {
	/// change status for this text
	fn set_status(self, status: Status) -> Self;
	/// change what you show in this text
	fn set_text(self, text: impl Into<String>) -> Self;
	/// get the color of this text, note if color is setted, color from status will unuse.
	fn get_color(&self, ui: &mut Ui) -> Color;
	/// change width of this text
	fn set_width(self, width: f32) -> Self;
	/// change height of this text
	fn set_height(self, height: f32) -> Self;
	/// change scale of this text
	fn set_scale(self, scale: Vec2) -> Self;
	/// change scale of this text by given em measure
	fn set_em(self, em: Vec2) -> Self;
	/// show or not show the underline
	fn underline(self, underline: bool) -> Self;
	/// get how large did this text take
	fn text_area(&self, painter: &mut Painter) -> Area;
	/// draw this text
	fn text_draw(&self, painter: &mut Painter, position: Vec2, ui: &mut Ui);
	/// change color for a text
	fn set_color(self, color: impl Into<Color>) -> Self;
	/// make current text bold or not
	fn set_bold(self, is_bold: bool) -> Self;
	/// make current text italic or not
	fn set_italic(self, is_italic: bool) -> Self;
}

impl TextSetting for Text {
	fn set_status(self, status: Status) -> Self {
		Self {
			status,
			..self
		}
	}

	fn set_text(self, text: impl Into<String>) -> Self{
		Self {
			text: text.into(),
			..self
		}
	}
	
	fn get_color(&self, ui: &mut Ui) -> Color {
		match self.color {
			Some(t) => t,
			None => self.status.into_color(ui)
		}
	}

	fn set_width(self, width: f32) -> Self {
		Self {
			width: Some(width),
			..self
		}
	}

	fn set_height(self, height: f32) -> Self {
		Self {
			height: Some(height),
			..self
		}
	}

	fn set_scale(self, size: Vec2) -> Self {
		Self {
			size,
			..self
		}
	}

	fn set_em(self, em: Vec2) -> Self {
		Self {
			size: em / Vec2::same(EM),
			..self
		}
	}

	fn underline(self, underline: bool) -> Self {
		Self {
			underline,
			..self
		}
	}

	fn text_draw(&self, painter: &mut Painter, position_given: Vec2, ui: &mut Ui) {
		let scale = painter.style().size;
		let position = painter.style().position;
		let color = painter.style().fill;
		let text_style = painter.text_style().clone();
		painter.set_text_style(self.style.clone());
		painter.set_color(self.get_color(ui));
		painter.set_position(position_given);
		painter.set_scale(self.size);
		if let Some(width) = self.width {
			if let Some(height) = self.height {
				painter.text_with_limit(self.text.clone(), width, height);
			}else {
				painter.text_with_width(self.text.clone(), width);
			}
		}else {
			painter.text(self.text.clone());
		}
		if self.underline {
			// TODO: make this changable
			let rect_height = 5.0;
			let text_area = self.text_area(painter);
			painter.set_position(text_area.left_bottom() + Vec2::new(0.0,2.0));
			painter.rect(Vec2::new(text_area.width(), rect_height), Vec2::same(rect_height * 0.5));
		};
		painter.set_scale(scale);
		painter.set_position(position);
		painter.set_color(color);
		painter.set_text_style(text_style);
	}

	fn text_area(&self, painter: &mut Painter) -> Area {
		let scale = painter.style().size;
		painter.set_scale(self.size);
		let back = painter.text_area(self.text.clone());
		painter.set_scale(scale);
		back
	}

	fn set_color(self, color: impl Into<Color>) -> Self {
		Self {
			color: Some(color.into()),
			..self
		}
	}

	fn set_bold(self, is_bold: bool) -> Self {
		Self {
			style: self.style.set_bold(is_bold),
			..self
		}
	}
	fn set_italic(self, is_italic: bool) -> Self { 
		Self {
			style: self.style.set_italic(is_italic),
			..self
		}
	}
}

impl<T> From<T> for Text where
	T: Into<String>
{
	fn from(value: T) -> Self {
		Text {
			text: value.into(),
			..Default::default()
		}
	}
}

/// a button
///
/// # Example
/// ```no_run
/// # use nablo::prelude::Button;
/// # let mut ui = nablo::Ui::default();
/// if ui.add(Button::new("Hello World")).is_clicked() {
///     println!("Hello World!");
/// }
/// ```
#[derive(Default)]
pub struct Button {
	text: Text,
	painter: Painter,
	space: Option<f32>,
	status: Status,
	style: ButtonStyle
}

impl Status {
	/// using what in [`Style`] to transform into color
	pub fn into_color(&self, ui: &mut Ui) -> Color {
		match self {
			Self::Error => ui.style().error_color,
			Self::Info => ui.style().info_color,
			Self::Warning => ui.style().warning_color,
			Self::Success => ui.style().success_color,
			Self::Default => ui.style().text_color,
		}
	}
}

/// a canvas
///
/// # Example
/// ```no_run
/// # use nablo::prelude::Canvas;
/// # use nablo::prelude::Vec2;
/// # let mut ui = nablo::Ui::default();
/// // just draw a rectangle
/// ui.add(Canvas::new(Vec2::new(200.0, 100.0), |painter| {
///     painter.rect(Vec2::new(200.0, 100.0), Vec2::same(0.0));
/// }));
/// ```
#[derive(Default)]
pub struct Canvas {
	painter: Painter,
	width_and_height: Vec2,
	dragable: bool,
}

/// a single line of text
///
/// # Example
/// ```no_run
/// # use nablo::prelude::Label;
/// # let mut ui = nablo::Ui::default();
/// ui.add(Label::new("We already walked too far, down to we had forgotten why embarked."));
/// ```
#[derive(Default)]
pub struct Label {
	text: Text,
}

/// a smlicated way to imply [`TextSetting`] for structs have text
macro_rules! imply_text_trait{
	($a:ty)=>{
		impl TextSetting for $a {
			fn set_status(self, status: Status) -> Self { Self { text: self.text.set_status(status), ..self } }
			fn get_color(&self, ui: &mut Ui) -> Color { self.text.get_color(ui) }
			fn set_width(self, width: f32) -> Self { Self { text: self.text.set_width(width), ..self } }
			fn set_height(self, height: f32) -> Self { Self { text: self.text.set_height(height), ..self } }
			fn set_scale(self, scale: Vec2) -> Self { Self { text: self.text.set_scale(scale), ..self } }
			fn set_em(self, em: Vec2) -> Self { Self { text: self.text.set_em(em), ..self } }
			fn underline(self, underline: bool) -> Self { Self { text: self.text.underline(underline), ..self } }
			fn text_area(&self, painter: &mut Painter) -> Area { self.text.text_area(painter) }
			fn text_draw(&self, painter: &mut Painter, position: Vec2, ui: &mut Ui) { self.text.text_draw(painter, position, ui) }
			fn set_text(self, text: impl Into<String>) -> Self { Self { text: self.text.set_text(text), ..self } }
			fn set_color(self, color: impl Into<Color>) -> Self { Self { text: self.text.set_color(color), ..self } }
			fn set_bold(self, is_bold: bool) -> Self { Self { text: self.text.set_bold(is_bold), ..self } }
			fn set_italic(self, is_italic: bool) -> Self { Self { text: self.text.set_italic(is_italic), ..self } }
		}
	}
}

/// one out of several alternatives, either selected or not. will mark selected items with a different background color.
#[derive(Default)]
pub struct SelectableValue {
	select: bool,
	text: Text,
	painter: Painter,
	space: Option<f32>,
	status: Status
}

/// a widgets that does nothing
pub struct Empty {
	pub width_and_height: Vec2,
}

impl Empty {
	/// an [`Empty`] that literally takes empty place
	pub const EMPTY: Empty = Empty {
		width_and_height: Vec2::ZERO
	};

	/// create a new [`Empty`]
	pub fn new(width_and_height: Vec2) -> Self {
		Self {
			width_and_height,
		}
	}
}

impl Widget for Empty {
	fn draw(&mut self, _: &mut Ui, _: &Response, _: &mut Painter) {}
	fn ui(&mut self, ui: &mut Ui, area: Option<Area>) -> Response { ui.response(area.unwrap_or([ui.available_position(), ui.available_position() + self.width_and_height].into()), false, false) }
}

/// a single line to input.
pub struct SingleTextInput<'a> {
	text: Text,
	input: &'a mut String,
	painter: Painter,
	width: Option<f32>,
	place_holder: String,
	is_password: bool,
	space: Option<f32>,
	limit: Option<usize>,
}

/// a simple slider
pub struct Slider<'a, T: Num> {
	input: &'a mut T,
	from: T,
	to: T,
	step: f64,
	is_logarithmic: bool,
	width: f32,
	speed: f64,
	text: Text,
	prefix: String,
	suffix: String,
}

/// a value can be draggable. TODO: text input
pub struct DragableValue<'a, T: Num> {
	input: &'a mut T,
	from: T,
	to: T,
	is_logarithmic: bool,
	step: f64,
	speed: f64,
	text: Text,
	prefix: String,
	suffix: String,
}

/// easy as it should be
pub struct DivideLine {
	is_horizental: bool,
	// to lazy to white a new enum to represent align....
	// None for center, true for left/top, false for right/bottom
	centered: Option<bool>
}

/// A sinple progress bar
pub struct ProgressBar {
	progress: f64,
	attach: bool,
	status: Status,
	width: f32
}

imply_text_trait!(SingleTextInput<'_>);
imply_text_trait!(Button);
imply_text_trait!(SelectableValue);
impl TextSetting for Label {
	fn set_status(self, status: Status) -> Self { Self { text: self.text.set_status(status) } }
	fn get_color(&self, ui: &mut Ui) -> Color { self.text.get_color(ui) }
	fn set_width(self, width: f32) -> Self { Self { text: self.text.set_width(width) } }
	fn set_height(self, height: f32) -> Self { Self { text: self.text.set_height(height)} }
	fn set_scale(self, scale: Vec2) -> Self { Self { text: self.text.set_scale(scale) } }
	fn set_em(self, em: Vec2) -> Self { Self { text: self.text.set_em(em)} }
	fn underline(self, underline: bool) -> Self { Self { text: self.text.underline(underline)} }
	fn text_area(&self, painter: &mut Painter) -> Area { self.text.text_area(painter) }
	fn text_draw(&self, painter: &mut Painter, position: Vec2, ui: &mut Ui) { self.text.text_draw(painter, position, ui) }
	fn set_text(self, text: impl Into<String>) -> Self { Self { text: self.text.set_text(text)} }
	fn set_color(self, color: impl Into<Color>) -> Self { Self { text: self.text.set_color(color)} }
	fn set_bold(self, is_bold: bool) -> Self { Self { text: self.text.set_bold(is_bold)} }
	fn set_italic(self, is_italic: bool) -> Self { Self { text: self.text.set_italic(is_italic)} }
}
imply_text_trait!(Collapsing);
imply_text_trait!(Message);
imply_text_trait!(TooltipProvider);

/// for a numeric value use in slider etc.
pub trait Num: PartialOrd + PartialEq + Sized + Clone {
	fn to_f64(&self) -> f64;
	fn from_f64(input: f64) -> Self;
}

macro_rules! impl_num {
	($t: ty) => {
		impl Num for $t {
			fn to_f64(&self) -> f64 { *self as f64 }
			fn from_f64(input: f64) -> Self { input as $t }
		}
	}
}

impl_num!(f32);
impl_num!(f64);
impl_num!(i8);
impl_num!(i16);
impl_num!(i32);
impl_num!(i64);
impl_num!(i128);
impl_num!(isize);
impl_num!(u8);
impl_num!(u16);
impl_num!(u32);
impl_num!(u64);
impl_num!(u128);
impl_num!(usize);