//! containers provided by `nablo`

mod card;
mod collapsing;
pub mod message_provider;

use nablo_shape::prelude::Painter;
use crate::prelude::Text;
use crate::prelude::shape_elements::Color;
use nablo_shape::math::Vec2;
use crate::widgets::Status;

/// the most basic container
#[derive(Default, Clone)]
pub struct Card {
	position: Option<Vec2>,
	width: Option<f32>,
	height: Option<f32>,
	id: String,
	rounding: Vec2,
	status: Status,
	color: Option<Color>,
	scrollable: [bool; 2],
	dragable: bool,
	resizable: bool,
	stroke_width: f32,
	stroke_color: Option<Color>,
}

/// show a collapsing area.
#[derive(Default, Clone)]
pub struct Collapsing {
	pub(crate) id: String,
	pub(crate) text: Text,
	pub(crate) icon: Option<Painter>,
	pub(crate) default_open: bool
}

/// show a message in yor app
#[derive(Default, Clone)]
pub struct MessageProvider {
	id: String
}
