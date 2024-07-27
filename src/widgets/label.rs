use crate::Response;
use crate::Widget;
use crate::widgets::Text;
use nablo_shape::math::Area;
use nablo_shape::shape::Painter;
use crate::Ui;
use crate::widgets::TextSetting;
use crate::widgets::Label;

impl Label {
	/// create a new lable
	pub fn new(text: impl Into<Text>) -> Self {
		Self {
			text: text.into().text.replace('\n', "").into(),
		}
	}
}

impl Widget for Label {
	fn draw(&mut self, ui: &mut Ui, response: &Response, painter: &mut Painter) {
		self.text_draw(painter, response.area.left_top(), ui);
	}

	fn ui(&mut self, ui: &mut Ui, area: Option<Area>) -> Response {
		let mut painter = ui.painter();
		let text_area = self.text_area(&mut painter);
		let area = match area {
			Some(t) => t,
			None => Area::new(ui.available_position(), ui.available_position() + text_area.width_and_height())
		};
		ui.response(area, false, false)
	}
}
