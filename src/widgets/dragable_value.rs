use std::ops::RangeInclusive;
use crate::prelude::Text;
use crate::widgets::Num;
use crate::prelude::DragableValue;
use nablo_shape::prelude::Animation;
use time::Duration;
use crate::Response;
use nablo_shape::prelude::Area;
use nablo_shape::prelude::Painter;
use crate::Vec2;
use crate::prelude::TextSetting;
use crate::prelude::Status;
use crate::Ui;
use crate::widgets::Color;
use crate::Widget;

impl<'a, T: Num> DragableValue<'a, T> {
	/// get a dragable value
	pub fn new(input: &'a mut T) -> Self {
		Self {
			text: "".into(),
			input,
			from: T::from_f64(f64::NEG_INFINITY),
			to: T::from_f64(f64::INFINITY),
			is_logarithmic: false,
			step: 1.0,
			speed: 1.0,
			suffix: "".into(),
			prefix: "".into(),
		}
	}

	/// set speed to dragable value
	pub fn speed(self, speed: f64) -> Self {
		Self {
			speed,
			..self
		}
	}

	/// set step to dragable value
	pub fn step(self, step: f64) -> Self {
		Self {
			step,
			..self
		}
	}

	/// set suffix to dragable value
	pub fn suffix(self,suffix: impl Into<String>) -> Self {
		Self {
			suffix: suffix.into(),
			..self
		}
	}

	/// set range to slider
	pub fn range(self, range: RangeInclusive<T>) -> Self {
		Self {
			from: range.start().clone(),
			to: range.end().clone(),
			..self
		}
	}

	/// set slider to logarithmic
	pub fn logarithmic(self, is_logarithmic: bool) -> Self {
		Self {
			is_logarithmic,
			..self
		}
	}

	/// set prefix to dragable value
	pub fn prefix(self,prefix: impl Into<String>) -> Self {
		Self {
			prefix: prefix.into(),
			..self
		}
	}
}

impl<T: Num> Widget for DragableValue<'_, T> {
	fn draw(&mut self, ui: &mut Ui, response: &Response, painter: &mut Painter) {
		painter.set_transform_origin(response.area.area[0]);
		// logic
		let from = self.from.to_f64();
		let to = self.to.to_f64();
		let step = self.step.to_f64();
		let speed = self.speed.to_f64();
		let compress = |input: f64| -> f64 {
			if input > to {
				to
			}else if input < from {
				from
			}else {
				input
			}
		};
		let drag_delta = response.drag_delta().x;
		let input = if from.is_infinite() || to.is_infinite() {
			let change = drag_delta as f64 * speed;
			((self.input.to_f64() + change) / step).round() * step
		}else if self.is_logarithmic {
			(drag_delta as f64 * speed * (to.ln() - from.ln()) + self.input.to_f64().ln()).exp()
		}else {
			let change = drag_delta as f64 * speed;
			((self.input.to_f64() + change) / step).round() * step
		};
		*self.input = T::from_f64(compress(input));

		// animation caculate
		let animation_time = Duration::milliseconds(250);
		let animation = Animation::new_standard(animation_time, Vec2::new(0.5, 0.0), Vec2::new(0.5, 1.0));
		let light_factor = if let Some(lost_hover_time) = response.lost_hovering_time() {
			if let Some(hover_time) = response.hovering_time(){
				if hover_time - lost_hover_time > animation_time {
					1.0 - animation.caculate(&lost_hover_time).unwrap_or(1.0)
				}else {
					animation.caculate(&(hover_time - lost_hover_time - lost_hover_time)).unwrap_or(0.0)
				}
			}else {
				0.0
			}
		}else if let Some(hover_time) = response.hovering_time() {
			animation.caculate(&hover_time).unwrap_or(1.0)
		}else {
			0.0
		} * ui.style().brighten_factor;

		// actual draw
		let origin = response.area.left_top();
		let background_color = ui.style().background_color.brighter(0.15);

		let input_text = format!("{}{}{}", self.prefix, self.input.to_f64(), self.suffix);
		let mut text: Text = input_text.into();
		let input_text_area = text.text_area(painter);
		let text_area = self.text.text_area(painter);

		painter.set_color(background_color);
		let space = if text_area.width() == 0.0 {
			0.0
		}else {
			text_area.width() + ui.style().space
		};
		painter.set_position(response.area.area[0] + Vec2::new(space + 2.0, 2.0));
		painter.set_stroke_color(ui.style.slider_unreached_color);
		painter.set_stroke_width(2.0);
		painter.rect(Vec2::new(input_text_area.width() + ui.style().space * 2.0, response.area.height()) - Vec2::same(2.0), Vec2::same(2.5));
		painter.set_position(response.area.area[0] + Vec2::new(space, 0.0));

		let width = if input_text_area.width() > response.area.width() + ui.style().space * 2.0 {
			response.area.width() + ui.style().space * 2.0
		}else {
			input_text_area.width() 
		};
		text = text.clone().set_width(width);
		text.text_draw(painter, origin + Vec2::new(ui.style().space + space, (response.area.height() - input_text_area.height()) / 2.0), ui);

		let width = if text_area.width() > response.area.width() + ui.style().space * 2.0 {
			response.area.width() + ui.style().space * 2.0
		}else {
			text_area.width() 
		};
		self.text = self.text.clone().set_width(width);
		self.text.text_draw(painter, origin + Vec2::new(0.0, (response.area.height() - text_area.height()) / 2.0), ui);

		// apply light efect
		painter.brighter(light_factor);
	}

	fn ui(&mut self, ui: &mut Ui, area: Option<Area>) -> Response {
		if let Some(area) = area {
			return ui.response(area, true, true);
		}
		let mut painter = ui.painter();
		let input_text = format!("{}{}{}", self.prefix, self.input.to_f64(), self.suffix);
		let text: Text = input_text.into();
		let fix_area = text.text_area(&mut painter);
		let text_area = self.text.text_area(&mut painter);
		let width =  text_area.width() + 3.0 * ui.style().space + fix_area.width();
		let height = [16.0, text_area.height(), fix_area.height()].iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap() + ui.style().space;
		let area = Area::new(ui.available_position(), ui.available_position() + Vec2::new(width, height));
		ui.response(area, true, true)
	}
}

impl<T: Num> TextSetting for DragableValue<'_, T> {
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