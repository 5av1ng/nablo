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
			step: T::from_f64(1.0),
			speed: T::from_f64(1.0),
			suffix: "".into(),
			prefix: "".into(),
			non_negative: false
		}
	}

	/// set speed to dragable value
	pub fn speed(self, speed: T) -> Self {
		Self {
			speed,
			..self
		}
	}

	/// set step to dragable value
	pub fn step(self, step: T) -> Self {
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

	/// set prefix to dragable value
	pub fn prefix(self,prefix: impl Into<String>) -> Self {
		Self {
			prefix: prefix.into(),
			..self
		}
	}

	/// set if this dragable value can contains negative value
	pub fn non_negative(self, non_negative: bool) -> Self {
		Self {
			non_negative,
			..self
		}
	}
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct DragableValueTemp {
	last_position: Option<Vec2>
}

impl<T: Num> Widget for DragableValue<'_, T> {
	fn draw(&mut self, ui: &mut Ui, response: &Response, painter: &mut Painter) {
		// logic
		let step = self.step.to_f64();
		let speed = self.speed.to_f64();
		let mut temp: DragableValueTemp = if let Some(t) = ui.memory_read(&response.id) {
			t
		}else {
			DragableValueTemp::default()
		};
		if let Some(t) = response.drag() {
			if let Some(l) = temp.last_position {
				let change = (t.x - l.x) as f64 * speed;
				let mut input = ((self.input.to_f64() + change) / step).round() * step;
				if self.non_negative && input < 0.0 {
					input = 0.0;
				}
				*self.input = T::from_f64(input);
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

		let input_text = format!("{}{}{}", self.prefix, self.input.to_f64(), self.suffix);
		let mut text: Text = input_text.into();
		let input_text_area = text.text_area(painter);
		let text_area = self.text.text_area(painter);

		painter.set_color(background_color);
		painter.set_position(response.area.area[0] + Vec2::new(text_area.width() + ui.style().space, 0.0));
		let stroke_color: Color = 1.0.into();
		let stroke_color = stroke_color.set_alpha(255);
		painter.set_stroke_color(stroke_color);
		painter.set_stroke_width(1.0);
		painter.rect(Vec2::new(input_text_area.width() + ui.style().space * 2.0, response.area.height()), Vec2::same(2.5));

		let width = if input_text_area.width() > response.area.width() + ui.style().space * 2.0 {
			response.area.width() + ui.style().space * 2.0
		}else {
			input_text_area.width() 
		};
		text = text.clone().set_width(width);
		text.text_draw(painter, origin + Vec2::new(ui.style().space * 2.0 + text_area.width(), (response.area.height() - input_text_area.height()) / 2.0), ui);

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
			return ui.response(area);
		}
		let mut painter = ui.painter();
		let input_text = format!("{}{}{}", self.prefix, self.input.to_f64(), self.suffix);
		let text: Text = input_text.into();
		let fix_area = text.text_area(&mut painter);
		let text_area = self.text.text_area(&mut painter);
		let width =  text_area.width() + 3.0 * ui.style().space + fix_area.width();
		let height = [16.0, text_area.height(), fix_area.height()].iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap() + ui.style().space;
		let area = Area::new(ui.available_position(), ui.available_position() + Vec2::new(width, height));
		ui.response(area)
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