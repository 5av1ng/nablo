use crate::Instant;
use nablo_shape::prelude::shape_elements::Color;
use crate::widgets::TextSetting;
use crate::widgets::Text;
use crate::widgets::SelectableValue;
use time::Duration;
use nablo_shape::shape::animation::Animation;
use nablo_shape::math::Area;
use nablo_shape::math::Vec2;
use crate::Ui;
use crate::Response;
use nablo_shape::shape::Painter;
use crate::Widget;

#[derive(Debug, serde::Serialize, serde::Deserialize, Default, Clone)]
struct SelectableValueTemp {
	last_select: bool,
	change_time: Vec<Instant>
}

impl SelectableValue {
	/// get a selectable value with text
	pub fn new(select: bool, text: impl Into<Text>) -> Self {
		Self {
			select: select,
			text: text.into(),
			..Default::default()
		}
	}

	/// set icon for a selectable value
	pub fn icon(self, area: Vec2, icon: impl FnOnce(&mut Painter)) -> Self {
		let mut painter = self.painter;
		painter.paint_area = Area::new(Vec2::ZERO, area);
		painter.set_clip(Area::new(Vec2::ZERO, area));
		icon(&mut painter);
		Self {
			painter,
			..self
		}
	}

	/// check if current selectable value have icon
	pub fn has_icon(&self) -> bool {
		!self.painter.is_empty()
	}

	/// set padding of each element
	pub fn set_padding(self, padding: f32) -> Self {
		Self {
			space: Some(padding),
			..self
		}
	}
}

impl Widget for SelectableValue {
	fn draw(&mut self, ui: &mut Ui, response: &Response, painter: &mut Painter) {
		let space = self.space.unwrap_or(ui.style().space);
		let background_color = ui.style().primary_color;
		self.painter.change_layer(painter.style().layer);
		// hover animation
		let animation_time = Duration::milliseconds(250);
		let animation = Animation::new_standard(animation_time, Vec2::new(0.3, 0.0), Vec2::new(0.7, 1.0));
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

		// switch animation
		let temp: Option<SelectableValueTemp> = response.memory_read();
		let alpha;
		if let Some(memory) = temp {
			let mut memory = memory;
			if self.select != memory.last_select {
				memory.change_time.push(Instant::now());
				if memory.change_time.len() > 2 {
					memory.change_time.remove(0);
				}
				memory.last_select = self.select;
				ui.memory_save(&response.id, memory.clone());
			}
			alpha = if memory.change_time.len() == 2 {
				let delta = memory.change_time[0].elapsed() - memory.change_time[1].elapsed();
				let calc = if delta > animation_time {
					animation.caculate(&memory.change_time[1].elapsed()).unwrap_or_else(|| 1.0)
				}else {
					animation.caculate(&(delta + memory.change_time[1].elapsed())).unwrap_or_else(|| 1.0)
				};
				if self.select {
					calc
				}else {
					1.0 - calc
				}
			}else if memory.change_time.len() == 1 {
				let calc = animation.caculate(&memory.change_time[0].elapsed()).unwrap_or_else(|| 1.0);
				if self.select {
					calc
				}else {
					1.0 - calc
				}
			}
			else {
				if self.select {
					1.0
				}else {
					0.0
				}
			};
		}else {
			ui.memory_save(&response.id, SelectableValueTemp {
				last_select: self.select,
				change_time: vec!()
			});
			alpha = 0.0;
		}

		// actual draw
		painter.set_color(ui.style().background_color.brighter(0.15) + alpha * (background_color - ui.style().background_color.brighter(0.15)));
		painter.set_position(response.area.area[0]);
		painter.rect(response.area.width_and_height(), Vec2::same(5.0));

		let icon_area = self.painter.paint_area;
		let position = response.area.area[0] + Vec2::new(space * 0.5, (response.area.height() - icon_area.height()) / 2.0);
		self.painter.move_delta_to(position);
		self.painter.change_clip(painter.style().clip);
		painter.append(&mut self.painter);

		let text_area = self.text.text_area(painter);
		let text_width = if text_area.width() < response.area.width() {
			response.area.width()
		}else {
			text_area.width()
		};
		let position = response.area.area[0] + Vec2::new(icon_area.width() + space, (response.area.height() - text_area.height()) / 2.0);
		self.text = self.text.clone().set_width(text_width);
		self.text.text_draw(painter, position, ui);

		// apply light efect
		painter.set_position(response.area.area[0]);
		painter.set_color(Color::WHITE.set_alpha((light_factor * 255.0) as u8));
		painter.rect(response.area.width_and_height(), Vec2::same(5.0));
	}

	fn ui(&mut self, ui: &mut Ui, area: Option<Area>) -> Response {
		let space = self.space.unwrap_or(ui.style().space);
		let mut painter = ui.painter();
		let text_area = self.text_area(&mut painter);
		let icon_area = self.painter.paint_area;
		let height = if icon_area.height() > text_area.height() {
			icon_area.height()
		}else {
			text_area.height()
		};
		let height = height + space;
		let width = text_area.width() + icon_area.width() + space * 2.0;
		let area = match area {
			Some(t) => t,
			None => Area::new(ui.available_position(), ui.available_position() + Vec2::new(width, height))
		};
		ui.response(area)
	}
}