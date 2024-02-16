use nablo_shape::prelude::Painter;
use nablo_shape::prelude::Vec2;
use nablo_shape::prelude::Area;
use crate::Ui;
use crate::Response;
use crate::Widget;
use crate::prelude::DivideLine;

impl DivideLine {
	/// get a new divide line with centered and horizental position
	pub fn new() -> Self {
		Self {
			is_horizental: true,
			centered: None
		}
	}

	/// set vertical to divide line
	pub fn vertical(self) -> Self {
		Self {
			is_horizental: false,
			..self
		}
	}

	/// set horizental to divide line
	pub fn horizental(self) -> Self {
		Self {
			is_horizental: true,
			..self
		}
	}

	/// set centered to divide line
	pub fn centered(self) -> Self {
		Self {
			centered: None, 
			..self
		}
	}

	/// set top/left to divide line
	pub fn top(self) -> Self {
		Self {
			centered: Some(true), 
			..self
		}
	}

	/// set bottom/right to divide line
	pub fn bottom(self) -> Self {
		Self {
			centered: Some(false), 
			..self
		}
	}
}

impl Widget for DivideLine {
	fn draw(&mut self, _: &mut Ui, response: &Response, painter: &mut Painter) {
		painter.set_color([1.0, 1.0, 1.0, 0.3]);
		if let Some(inner) = self.centered {
			if inner {
				painter.set_position(response.area.area[0]);
				if self.is_horizental {
					painter.rect(Vec2::new(response.area.width(), 4.0), Vec2::same(2.0));
				}else {
					painter.rect(Vec2::new(4.0 , response.area.height()), Vec2::same(2.0));
				}
			}else {
				if self.is_horizental {
					painter.set_position(Vec2::new(response.area.area[0].x, response.area.area[1].y - 4.0));
					painter.rect(Vec2::new(response.area.width(), 4.0), Vec2::same(2.0));
				}else {
					painter.set_position(Vec2::new(response.area.area[1].x - 4.0 , response.area.area[0].y));
					painter.rect(Vec2::new(4.0 , response.area.height()), Vec2::same(2.0));
				}
			}
		}else {
			if self.is_horizental {
				painter.set_position(response.area.area[0] + Vec2::y(response.area.height() / 2.0 - 2.0));
				painter.rect(Vec2::new(response.area.width(), 4.0), Vec2::same(2.0));
			}else {
				painter.set_position(response.area.area[0] + Vec2::x(response.area.width() / 2.0 - 2.0));
				painter.rect(Vec2::new(4.0 , response.area.height()), Vec2::same(2.0));
			}
		}
	}
	fn ui(&mut self, ui: &mut Ui, area: Option<Area>) -> Response {
		let area = match area {
			Some(t) => t,
			None => {
				if self.is_horizental {
					Area::new(ui.available_position(), ui.available_position() + Vec2::new(ui.window_area().right_top().x - ui.available_position().x - ui.style().space, ui.style().space))
				}else {
					Area::new(ui.available_position(), ui.available_position() + Vec2::new(ui.style().space, ui.window_area().height() - ui.available_position().y + ui.window_area().left_top().y - ui.style().space))
				}
			}
		};
		ui.response(area)
	}
}