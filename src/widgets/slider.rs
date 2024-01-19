use crate::prelude::Num;
use nablo_shape::prelude::Animation;
use time::Duration;
use crate::prelude::Text;
use std::ops::RangeInclusive;
use crate::Response;
use nablo_shape::prelude::Area;
use nablo_shape::prelude::Painter;
use crate::Vec2;
use crate::prelude::TextSetting;
use crate::prelude::Status;
use crate::Ui;
use crate::widgets::Color;
use crate::Widget;
use crate::prelude::Slider;

impl<'a, T: Num> Slider<'a, T> {
	/// get a slider with text
	pub fn new(range: RangeInclusive<T> ,input: &'a mut T) -> Self {
		Self {
			text: "".into(),
			input,
			from: range.start().clone(),
			to: range.end().clone(),
			step: T::from_f64(1.0),
			is_logarithmic: false,
			prefix: "".into(),
			speed: T::from_f64(1.0),
			suffix: "".into(),
			width: 100.0,
		}
	}

	/// set slider to logarithmic
	pub fn logarithmic(self, is_logarithmic: bool) -> Self {
		Self {
			is_logarithmic,
			..self
		}
	}

	/// set speed to slider
	pub fn speed(self, speed: T) -> Self {
		Self {
			speed,
			..self
		}
	}

	/// set step to slider
	pub fn step(self, step: T) -> Self {
		Self {
			step,
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

	/// set width to slider
	pub fn width(self, width: f32) -> Self {
		Self {
			width,
			..self
		}
	}

	/// set prefix to slider
	pub fn prefix(self,prefix: impl Into<String>) -> Self {
		Self {
			prefix: prefix.into(),
			..self
		}
	}

	/// set suffix to slider
	pub fn suffix(self,suffix: impl Into<String>) -> Self {
		Self {
			suffix: suffix.into(),
			..self
		}
	}
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct SliderTemp {
	last_position: Option<Vec2>
}

impl<T: Num> Widget for Slider<'_, T> {
	fn draw(&mut self, ui: &mut Ui, response: &Response, painter: &mut Painter) {
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
		let mut temp: SliderTemp = if let Some(t) = ui.memory_read(&response.id) {
			t
		}else {
			SliderTemp::default()
		};
		if let Some(t) = response.drag() {
			if let Some(l) = temp.last_position {
				let change = if self.is_logarithmic {
					// todo
					- ((t.x - l.x) / self.width) as f64 * speed * (from - to)
				}else {
					- ((t.x - l.x) / self.width) as f64 * speed * (from - to)
				};
				let input = ((self.input.to_f64() + change) / step).round() * step;
				*self.input = T::from_f64(compress(input));
			}
		}
		temp.last_position = response.drag();
		ui.memory_save(&response.id, temp);

		// animation caculate
		let animation_time = Duration::milliseconds(250);
		let animation = Animation::new_standard(animation_time, Vec2::new(0.5, 0.0), Vec2::new(0.5, 1.0));
		let light_factor = if let Some(lost_hover_time) = response.lost_hovering_time() {
			if let Some(hover_time) = response.hovering_time(){
				if hover_time - lost_hover_time > animation_time {
					1.0 - animation.caculate(&lost_hover_time).unwrap_or_else(|| 1.0)
				}else {
					animation.caculate(&(hover_time - lost_hover_time - lost_hover_time)).unwrap_or_else(|| 0.0)
				}
			}else {
				0.0
			}
		}else if let Some(hover_time) = response.hovering_time() {
			animation.caculate(&hover_time).unwrap_or_else(|| 1.0)
		}else {
			0.0
		} * ui.style().brighten_factor;

		// actual draw
		let origin = response.area.left_top();
		let background_color = ui.style().background_color.brighter(0.15);
		let text_area = self.text.text_area(painter);

		let inner_f64 = if self.input.to_f64() > to {
			to
		}else if self.input.to_f64() < from {
			from
		}else {
			self.input.to_f64()
		};
		let cir_x = ((inner_f64 - from) / (to - from)) as f32 * self.width + text_area.width() + ui.style().space - 8.0;
		let cir_y = (response.area.height() - 16.0) / 2.0;
		painter.set_position(origin + Vec2::new(text_area.width() + ui.style().space, cir_y + 4.0));
		painter.set_color(background_color);
		painter.rect(Vec2::new(self.width, 8.0), Vec2::same(4.0));
		painter.set_position(origin + Vec2::new(cir_x, cir_y));
		painter.set_color(background_color.brighter(0.1));
		painter.cir(8.0);

		let input_text: Text = format!("{}{}{}", self.prefix, inner_f64, self.suffix).into();
		let input_area = input_text.text_area(painter);
		let text_area = self.text.text_area(painter);
		let position = origin + Vec2::new(self.width + text_area.width() + 2.0 * ui.style().space, response.area.height() / 2.0 - input_area.height() / 2.0);
		input_text.text_draw(painter, position, ui);
		self.text.text_draw(painter, origin + Vec2::new(0.0, response.area.height() / 2.0 - text_area.height() / 2.0), ui);

		// apply light efect
		painter.brighter(light_factor);
	}

	fn ui(&mut self, ui: &mut Ui, area: Option<Area>) -> Response {
		if let Some(area) = area {
			return ui.response(area);
		}
		let mut painter = ui.painter();
		let input_text: Text = format!("{}{}{}", self.prefix, self.input.to_f64(), self.suffix).into();
		let input_text = input_text.text_area(&mut painter);
		let text_area = self.text.text_area(&mut painter);
		let width = 2.0 * ui.style().space + input_text.width() + text_area.width() + self.width;
		let height = [16.0, input_text.height(), text_area.height()].iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap() + ui.style().space;
		let area = Area::new(ui.available_position(), ui.available_position() + Vec2::new(width, height));
		ui.response(area)
	}
}

impl<T: Num> TextSetting for Slider<'_, T> {
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