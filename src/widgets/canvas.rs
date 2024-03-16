use nablo_shape::math::Vec2;
use crate::Response;
use crate::Ui;
use nablo_shape::shape::Painter;
use nablo_shape::math::Area;
use crate::Widget;
use crate::widgets::Canvas;

impl Widget for Canvas {
	fn draw(&mut self, ui: &mut Ui, response: &Response, painter: &mut Painter) {
		self.painter.change_layer(painter.style().layer);
		self.painter.move_delta_to(response.area.left_top());
		self.painter.change_clip(ui.window_crossed().shrink(Vec2::same(ui.style().space)).cross_part(&response.area));
		*painter = self.painter.clone();
	}

	fn ui(&mut self, ui: &mut Ui, area: Option<Area>) -> Response {
		let area = match area {
			Some(t) => t,
			None => Area::new(ui.available_position(), ui.available_position() + self.width_and_height)
		};
		ui.response(area, true, self.dragable)
	}
}

impl Canvas {
	/// get a new canvas
	pub fn new<P: FnOnce(&mut Painter)>(width_and_height: Vec2, paint: P) -> Self {
		let mut painter = Painter::from_area(&Area::new(Vec2::same(0.0), width_and_height));
		paint(&mut painter);
		Self {
			width_and_height,
			painter,
			dragable: false,
		}
	}

	/// get a new canvas, and you may want get return value from the painter function
	pub fn new_with_return<P: FnOnce(&mut Painter) -> R, R>(width_and_height: Vec2, paint: P) -> (Self, R) {
		let mut painter = Painter::from_area(&Area::new(Vec2::same(0.0), width_and_height));
		let back = paint(&mut painter);
		(Self {
			width_and_height,
			painter,
			dragable: false,
		}, back)
	}

	pub fn dragable(self, dragable: bool) -> Self {
		Self {
			dragable,
			..self
		}
	}
}