//! containers provided by `nablo`

mod card;
mod collapsing;
mod tooltip_provider;
pub mod message_provider;

use crate::prelude::Status;
use time::Duration;
use nablo_shape::prelude::Painter;
use crate::prelude::Text;
use crate::prelude::shape_elements::Color;
use nablo_shape::math::Vec2;
/// the most basic container
#[derive(Default, Clone)]
pub struct Card {
	position: Option<Vec2>,
	width: Option<f32>,
	height: Option<f32>,
	id: String,
	rounding: Vec2,
	// status: Status,
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

/// show a tool tip for a [`crate::prelude::Widget`]
#[derive(Clone)]
pub struct TooltipProvider {
	pub(crate) id: String,
	pub(crate) text: Text,
	pub(crate) align: [Align; 2],
	pub(crate) hover_time: Duration,
	// pub(crate) is_click_trigger: bool,
	pub(crate) space: Option<f32>,
	pub(crate) status: Status,
}

impl Default for TooltipProvider {
	fn default() -> Self {
		Self {
			id: "".into(),
			text: "".into(),
			align: [Align::Middle, Align::Middle],
			hover_time: Duration::ZERO,
			// is_click_trigger: false,
			space: None,
			status: Default::default(),
		}
	}
}

/// as name says
#[derive(Default, Clone)]
pub enum Align {
	/// or top
	Left,
	#[default] Middle,
	/// or bottom
	Right,
}
