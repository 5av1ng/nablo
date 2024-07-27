use crate::prelude::ButtonStyle;
use nablo_shape::prelude::shape_elements::Color;
use crate::prelude::Status;
use crate::widgets::TextSetting;
use crate::widgets::Text;
use crate::widgets::Button;
use time::Duration;
use nablo_shape::shape::animation::Animation;
use nablo_shape::math::Area;
use nablo_shape::math::Vec2;
use crate::Ui;
use crate::Response;
use nablo_shape::shape::Painter;
use crate::Widget;

impl Button {
	/// get a button with text
	pub fn new(text: impl Into<Text>) -> Self {
		Self {
			text: text.into(),
			..Default::default()
		}
	}

	/// get a button with nothing
	pub fn empty() -> Self {
		Self::default()
	}

	/// set icon for a button
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

	/// check if current button have icon
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

	/// set status
	pub fn status(self, status: Status) -> Self {
		Self {
			status,
			..self
		}
	}

	/// set style
	pub fn style(self, style: ButtonStyle) -> Self {
		Self {
			style,
			..self
		}
	}
}

impl Widget for Button {
	fn draw(&mut self, ui: &mut Ui, response: &Response, painter: &mut Painter) {
		let background_color = if let Status::Default = self.status {
			if ButtonStyle::Normal != self.style {
				self.status.into_color(ui)
			}else {
				ui.style().primary_color
			}
		}else {
			self.status.into_color(ui)
		};
		if self.text.color.is_none() {
			self.text.color = Some(
				if ButtonStyle::Normal == self.style {
					if background_color.difference(&Color::from(1.0)) > background_color.difference(&Color::from(0.0)) {
						1.0.into()
					}else {
						0.0.into()
					}
				}else {
					background_color
				}
			);
		}
		let space = self.space.unwrap_or(ui.style().space);
		self.painter.change_layer(painter.style().layer);
		// hover animation
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

		painter.set_transform_origin(response.area.area[0]);
		// click animation
		for ct in response.release_times() {
			let alpha = (1.0 - animation.caculate(&ct).unwrap_or(1.0)) * match self.style {
				ButtonStyle::Normal => 1.0,
				ButtonStyle::Stroked => 0.5,
				ButtonStyle::Lined => 0.1,
			};
			let offset = match self.style {
				ButtonStyle::Normal => (1.0 - alpha) * Vec2::same(5.0),
				ButtonStyle::Stroked => (0.5 - alpha) * Vec2::same(10.0),
				ButtonStyle::Lined => Vec2::same(0.0),
			};
			painter.set_position(response.area.area[0] + offset);
			let mut color_animation = background_color;
			color_animation.a = alpha;
			painter.set_color(color_animation);
			painter.rect(response.area.width_and_height(), Vec2::same(5.0));
		}

		// actual draw
		match self.style {
			ButtonStyle::Normal => {
				painter.set_color(background_color);
				painter.set_position(response.area.area[0]);
				painter.rect(response.area.width_and_height(), Vec2::same(5.0));
			},
			ButtonStyle::Stroked => {
				painter.set_stroke_color(background_color);
				let stroke_width = 1.0;
				painter.set_stroke_width(stroke_width);
				painter.set_color(Color::TRANSPARENT);
				painter.set_position(response.area.area[0] + Vec2::same(stroke_width));
				painter.rect(response.area.width_and_height() - Vec2::same(stroke_width * 2.0), Vec2::same(5.0));
			},
			ButtonStyle::Lined => {
				let text_area = self.text.text_area(painter);
				let icon_area = self.painter.paint_area;
				let length = (response.area.width() - space * 2.0 - icon_area.width()) * light_factor / ui.style().brighten_factor / 2.0;
				let position_l = response.area.area[0] + Vec2::new(response.area.width() / 2.0 - length, text_area.height() + 8.0);
				let position_r = response.area.area[0] + Vec2::new(response.area.width() / 2.0,  text_area.height() + 8.0);
				painter.set_color(background_color);
				painter.set_position(position_l);
				painter.rect(Vec2::new(length, 4.0), Vec2::same(2.0));
				painter.set_position(position_r);
				painter.rect(Vec2::new(length, 4.0), Vec2::same(2.0));
			},
		}
		let icon_area = self.painter.paint_area;
		let position = response.area.area[0] + Vec2::new(space / 2.0, (response.area.height() - icon_area.height()) / 2.0);
		self.painter.move_by(position);
		self.painter.change_clip(painter.style().clip);
		self.painter.scale_factor(painter.style().scale_factor);
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
		painter.brighter(light_factor);
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
		ui.response(area, true, false)
	}
}