/*! if you want to integrate `nablo`. here's what you need.
 * 
 * # Example
 * ```no_run
 * # use nablo::event::Event;
 * use nablo::integrator::*;
 * 
 * let mut integrator = Integrator::default();
 * 
 * loop {
 *     let events = gather_events();
 *     let output = integrator.frame(events, |ui| {
 *         // use ui to create something...
 *     });
 *     handle_output(output);
 * }
 * # fn gather_events() -> Vec<Event> { vec!() }
 * # fn handle_output<T>(_: T) {}
 * ```
*/

#[cfg(feature = "vertexs")]
use nablo_shape::prelude::Area;
#[cfg(feature = "vertexs")]
use nablo_shape::prelude::shape_elements::Image;
use crate::OutputEvent;
use nablo_shape::prelude::shape_elements::Color;
#[cfg(feature = "vertexs")]
use nablo_shape::prelude::shape_elements::Vertex;
#[cfg(feature = "vertexs")]
use nablo_shape::prelude::shape_elements::Style;
#[cfg(feature = "vertexs")]
use nablo_shape::prelude::shape_elements::Text;
use crate::Event;
use crate::Shape;
use crate::Ui;

/// the shape prased
#[cfg(feature = "vertexs")]
#[derive(Clone)]
pub enum ParsedShape {
	Vertexs {
		vertexs: Vec<Vertex>, 
		indices: Vec<u32>,
		clip_area: Area,
	},
	Text(Text, Style),
	Image(Image, Style),
}

/// helper of integrating
#[derive(Default)]
pub struct Integrator {
	/// you may want using this when dealing with some events.
	pub ui: Ui
}

/// after running ui code, here's things you should handle
pub struct Output<S> {
	/// background color, usually you will ignore it.
	pub background_color: Color,
	/// shapes you should draw. type of this value depends on what function you call. see more in [`Integrator`]
	pub shapes: S,
	/// the events you should handle, such as creating a texture
	pub output_events: Vec<OutputEvent>
}

impl Integrator {
	/// run the ui code for one frame.
	pub fn frame(&mut self, input_events: Vec<Event>, ui_code: impl FnOnce(&mut Ui)) -> Output<Vec<Shape>> {
		for event in input_events {
			self.ui.event(&event)
		}
		self.ui.update();
		ui_code(&mut self.ui);
		self.ui.raw_shape();
		let output = Output {
			background_color: self.ui.style().background_color,
			shapes: self.ui.shape.raw_shape.clone(),
			output_events: self.ui.output_events.clone(),
		};
		self.ui.clear();
		output
	}

	#[cfg(feature = "vertexs")]
	/// run the ui code for one frame, but out puts vertexs. dont take accout in texts
	pub fn frame_vertexs(&mut self, input_events: Vec<Event>, ui_code: impl FnOnce(&mut Ui)) -> Output<Vec<ParsedShape>> {
		for event in input_events {
			self.ui.event(&event)
		}
		self.ui.update();
		ui_code(&mut self.ui);
		self.ui.handle_raw_shape();
		let output = Output {
			background_color: self.ui.style().background_color,
			shapes: self.ui.shape.parsed_shapes.clone(),
			output_events: self.ui.output_events.clone(),
		};
		self.ui.clear();
		output
	}

	/// update for a single event
	pub fn event(&mut self, input_event: &Event) {
		self.ui.event(input_event)
	}
}