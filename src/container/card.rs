use nablo_shape::prelude::Animation;
use time::Duration;
use crate::Instant;
use crate::Response;
use crate::InnerResponse;
use nablo_shape::prelude::shape_elements::Layer;
use nablo_shape::prelude::shape_elements::Color;
use crate::container::Status;
use crate::Container;
use nablo_shape::math::Vec2;
use nablo_shape::math::Area;
use crate::Ui;
use crate::Painter;
use crate::container::Card;

#[derive(serde::Deserialize, serde::Serialize, Default, Debug)]
struct CardTemp {
	target: Vec2,
	from: Vec2,
	change_time: Vec<Instant>,
	maxium: Vec2,
	position: Option<Vec2>,
	size: Option<Vec2>,
	is_draging: bool,
	is_scrolling: bool,
	is_resizing: bool,
}

impl CardTemp {
	fn current(&mut self) -> Vec2 {
		if self.target == self.from {
			return self.target
		}
		let animation_time = Duration::milliseconds(250);
		let animation = Animation::new_standard(animation_time, Vec2::new(0.1, 0.5), Vec2::new(0.75, 1.0));
		let time = if self.change_time.is_empty() {
			self.change_time.push(Instant::now());
			Duration::milliseconds(0)
		}else {
			self.change_time[0].elapsed()
		};
		if let Some(t) = animation.caculate(&time) {
			(self.target - self.from) * t + self.from
		}else {
			self.from = self.target;
			self.target
		}
	}

	fn change(&mut self, target: Vec2, container_area: &Area) {
		let mut target = target;
		let maxium = self.maxium - container_area.width_and_height();
		if maxium.x < 0.0 {
			target.x = 0.0;
		}else {
			if target.x < - (maxium.x + 32.0) {
				target.x = - (maxium.x + 32.0);
			}else if target.x > 0.0 {
				target.x = 0.0;
			}
		}
		if maxium.y < 0.0 {
			target.y = 0.0;
		}else {
			if target.y < - maxium.y - 32.0 {
				target.y = - maxium.y - 32.0
			}else if target.y > 0.0 {
				target.y = 0.0;
			}
		}
		let current = self.current();
		self.target = target;
		self.change_time = vec!(Instant::now());
		self.from = current;
	}
}

impl Container for Card {
	fn get_id(&self, _: &mut Ui) -> String { self.id.clone() }
	fn area(&self, ui: &mut Ui) -> Area { self.get_area(ui) }
	fn layer(&self, _: &mut Ui) -> Layer { Layer::Bottom }
	fn begin(&mut self, ui: &mut Ui, painter: &mut Painter, response: &Response, id: &String) -> bool {
		if let Some(t) = self.color {
			painter.set_color(t)
		}else {
			painter.set_color(self.status.into_color(ui))
		}
		if let Some(t) = self.stroke_color {
			painter.set_stroke_width(self.stroke_width);
			painter.set_stroke_color(t);
		}
		painter.set_position(response.area.left_top());
		painter.rect(Vec2::new(response.area.width(), response.area.height()), self.rounding);
		painter.set_position(Vec2::ZERO);
		let mut temp: CardTemp = ui.memory_read(id).unwrap_or(CardTemp::default());
		painter.set_stroke_width(0.0);
		painter.set_stroke_color(1.0);
		painter.set_offset(temp.current());
		painter.set_clip(response.area.shrink(Vec2::same(ui.style().space)).cross_part(&ui.window_crossed()));
		true
	}
	fn end<R>(&mut self, ui: &mut Ui, painter: &mut Painter, inner_response: &InnerResponse<R>, id: &String) {
		let mut temp: CardTemp = ui.memory_read(id).unwrap_or(CardTemp::default());
		let cursor = ui.input().cursor_position().unwrap_or(Vec2::INF);
		if self.resizable {
			let inner_area = inner_response.response.area;
			let resize = vec!(Area::new(inner_area.left_top(), inner_area.left_top() + Vec2::same(16.0)),
				Area::new(inner_area.left_bottom() + Vec2::new(0.0, - 16.0), inner_area.left_bottom() + Vec2::new(16.0, 0.0)),
				Area::new(inner_area.right_bottom() - Vec2::same(16.0), inner_area.right_bottom()),
				Area::new(inner_area.right_top() + Vec2::new(- 16.0, 0.0), inner_area.right_top() + Vec2::new(0.0, 16.0))
			);
			let resize: Vec<bool> = resize.iter().map(|inner| inner.is_point_inside(&cursor)).collect();
			let resize = if (resize.contains(&true) || temp.is_resizing) && !temp.is_scrolling && !temp.is_draging {
				if let Some(_) = inner_response.response.drag() {
					temp.is_resizing = true;
					inner_response.response.drag_delta()
				}else {
					temp.is_resizing = false;
					Vec2::ZERO
				}
			}else {
				Vec2::ZERO
			};
			let mut size = temp.size.unwrap_or(inner_area.width_and_height()) + resize;
			if size.x < self.width.unwrap_or(f32::NEG_INFINITY) {
				size.x = self.width.unwrap_or(f32::NEG_INFINITY)
			}
			if size.y < self.height.unwrap_or(f32::NEG_INFINITY) {
				size.y = self.height.unwrap_or(f32::NEG_INFINITY)
			}
			temp.size = Some(size);
		}
		if self.dragable {
			let drag = vec!(Area::new(inner_response.response.area.left_top() + Vec2::new(16.0, 0.0), inner_response.response.area.right_top() + Vec2::new(-16.0, 16.0)),
				Area::new(inner_response.response.area.left_bottom() + Vec2::new(16.0, -16.0), inner_response.response.area.right_bottom() + Vec2::new(-16.0, 0.0)),
				Area::new(inner_response.response.area.left_top() + Vec2::new(0.0, 16.0), inner_response.response.area.left_bottom() + Vec2::new(16.0, -16.0)),
				Area::new(inner_response.response.area.right_top() + Vec2::new(-16.0, 16.0), inner_response.response.area.right_bottom() + Vec2::new(0.0, -16.0))
			);
			let drag: Vec<bool> = drag.iter().map(|inner| inner.is_point_inside(&cursor)).collect();
			let drag = if (drag.contains(&true) || temp.is_draging) && !temp.is_scrolling && !temp.is_resizing {
				if let Some(_) = inner_response.response.drag() {
					temp.is_draging = true;
					inner_response.response.drag_delta()
				}else {
					temp.is_draging = false;
					Vec2::ZERO
				}
			}else {
				Vec2::ZERO
			};
			temp.position = Some(temp.position.unwrap_or(inner_response.response.area.left_top()) + drag);
		};
		let mut area = Area::ZERO;
		for res in &inner_response.inner_responses {
			if res.area != inner_response.response.area {
				area.combine(&res.area);
			}
		}
		if self.scrollable[0] {
			temp.maxium.x = area.width_and_height().x;
		}else {
			temp.maxium.x = 0.0
		}
		if self.scrollable[1] {
			temp.maxium.y = area.width_and_height().y;
		}else {
			temp.maxium.y = 0.0
		}
		if (inner_response.response.area.shrink(Vec2::same(16.0)).is_point_inside(&cursor) && !temp.is_draging) || temp.is_scrolling {
			let scroll = if ui.input().scroll() != Vec2::ZERO {
				temp.is_scrolling = true;
				ui.input().scroll()
			}else {
				if let Some(_) = inner_response.response.drag() {
					temp.is_scrolling = true;
					inner_response.response.drag_delta()
				}else {
					temp.is_scrolling = false;
					Vec2::ZERO
				}
			};
			if scroll != Vec2::ZERO {
				if self.scrollable[0] {
					let scroll_inner = Vec2::new(temp.target.x + scroll.x, temp.target.y);
					temp.change(scroll_inner, &inner_response.response.area);
				}
				if self.scrollable[1] {
					let scroll_inner = Vec2::new(temp.target.x, temp.target.y + scroll.y);
					temp.change(scroll_inner, &inner_response.response.area);
				}
			}
		}
		ui.memory_save(id, &temp);
		let inner_area = inner_response.response.area.shrink(Vec2::same(ui.style().space));
		painter.set_clip(inner_response.response.area.cross_part(&ui.window_crossed()));
		if self.scrollable[0] {
			if area.width_and_height().x > inner_area.width() {
				let current = temp.current();
				let position = inner_response.response.area.left_bottom() + Vec2::new(ui.style().space, -10.0);
				painter.set_position(position);
				painter.set_color(ui.style().background_color.brighter(-0.05));
				painter.rect(Vec2::new(inner_area.width(), 5.0), Vec2::same(2.5));
				let length = (inner_area.width()).powf(2.0) / area.width_and_height().x;
				let x = (inner_area.width() - length) * current.x / (area.width_and_height().x - inner_area.width());
				let position = inner_response.response.area.left_bottom() + Vec2::new(ui.style().space, -10.0) + Vec2::new(-x, 0.0);
				painter.set_position(position);
				painter.set_color(ui.style().primary_color);
				painter.rect(Vec2::new(length, 5.0), Vec2::same(2.5));
			}
		}
		if self.scrollable[1] {
			if area.width_and_height().y > inner_area.height() {
				let current = temp.current();
				let position = inner_response.response.area.right_top() + Vec2::new(-10.0, ui.style().space);
				painter.set_position(position);
				painter.set_color(ui.style().background_color.brighter(-0.05));
				painter.rect(Vec2::new(5.0, inner_area.height()), Vec2::same(2.5));
				let length = (inner_area.height() ).powf(2.0) / area.width_and_height().y;
				let y = (inner_area.height() - length) * current.y / (area.width_and_height().y - inner_area.height());
				let position = inner_response.response.area.right_top() + Vec2::new(-10.0, ui.style().space) + Vec2::new(0.0, -y);
				painter.set_position(position);
				painter.set_color(ui.style().primary_color);
				painter.rect(Vec2::new(5.0, length), Vec2::same(2.5));
			}
		}
	}
	fn is_clickable(&self, _: &mut Ui) -> bool { true }
	fn is_dragable(&self, _: &mut Ui) -> bool { self.resizable || self.dragable || self.scrollable.contains(&true) }
}

impl Card {
	/// create a new card
	pub fn new(id: impl Into<String>) -> Self {
		Self {
			id: id.into(),
			..Default::default()
		}
	}

	/// set rounding to a card
	pub fn set_rounding(self, rounding: Vec2) -> Self {
		Self {
			rounding,
			..self
		}
	}

	/// set position to a card
	pub fn set_position(self, position: Vec2) -> Self {
		Self {
			position: Some(position),
			..self
		}
	}

	/// set height to a card, if the card is resizable, this will be minimal.
	pub fn set_height(self, height: f32) -> Self {
		Self {
			height: Some(height),
			..self
		}
	}

	/// set width to a card, if the card is resizable, this will be minimal.
	pub fn set_width(self, width: f32) -> Self {
		Self {
			width: Some(width),
			..self
		}
	}

	/// set size to a card
	pub fn set_size(self, size: Vec2) -> Self {
		Self {
			width: Some(size.x),
			height: Some(size.y), 
			..self
		}
	}

	/// set color to a card
	pub fn set_color(self, color: impl Into<Color>) -> Self {
		Self {
			color: Some(color.into()),
			..self
		}
	}

	/// set status to a card
	pub fn set_status(self, status: Status) -> Self {
		Self {
			status,
			..self
		}
	}

	/// set if this card can be drag to some place, can be used to simulate a window
	pub fn set_dragable(self, dragable: bool) -> Self {
		Self {
			dragable,
			..self
		}
	}

	/// set scrollable to a card
	pub fn set_scrollable(self, scroll: [bool; 2]) -> Self {
		Self {
			scrollable: scroll,
			..self
		}
	}

	/// set scrollable in x axis to a card
	pub fn set_scrollable_x(self, scroll: bool) -> Self {
		let binding = self.scrollable[1];
		self.set_scrollable([scroll, binding])
	}

	/// set scrollable in y axis to a card
	pub fn set_scrollable_y(self, scroll: bool) -> Self {
		let binding = self.scrollable[0];
		self.set_scrollable([binding, scroll])
	}

	/// set resizable to a card
	pub fn set_resizable(self, resizable: bool) -> Self {
		Self {
			resizable,
			..self
		}
	}

	/// set stroke width to a card
	pub fn set_stroke_width(self, stroke_width: f32) -> Self {
		Self {
			stroke_width,
			..self
		}
	}

	/// set stroke color to a card
	pub fn set_stroke_color(self, stroke_color: impl Into<Color>) -> Self {
		Self {
			stroke_color: Some(stroke_color.into()),
			..self
		}
	}

	/// where have we scrolled to?
	pub fn scroll(&self, ui: &mut Ui) -> Vec2 {
		let id = ui.container_id(self);
		let mut temp: CardTemp = ui.memory_read(&id).unwrap_or(CardTemp::default());
		temp.current()
	}

	/// where have we scrolled to in x axis?
	pub fn scroll_x(&self, ui: &mut Ui) -> f32 {
		self.scroll(ui).x
	}

	/// where have we scrolled to in y axis?
	pub fn scroll_y(&self, ui: &mut Ui) -> f32 {
		self.scroll(ui).y
	}

	/// scroll to some place, will do nothing if its not scrollable.
	pub fn scroll_to(&mut self, scroll: Vec2, ui: &mut Ui) {
		self.scroll_to_x(scroll.x, ui);
		self.scroll_to_y(scroll.y, ui);
	}

	/// scroll to some place in x axis exactly, will do nothing if its not scrollable.
	pub fn scroll_to_x(&mut self, scroll: f32, ui: &mut Ui) {
		let scroll = - scroll;
		if self.scrollable[0] {
			let id = ui.container_id(self);
			let area = self.area(ui);
			let mut temp: CardTemp = ui.memory_read(&id).unwrap_or(CardTemp::default());
			let scroll = Vec2::new(scroll, temp.target.y);
			temp.change(scroll, &area);
			ui.memory_save(&id, temp);
		}
	}

	/// scroll to some place in y axis exactly, will do nothing if its not scrollable.
	pub fn scroll_to_y(&mut self, scroll: f32, ui: &mut Ui) {
		let scroll = - scroll;
		if self.scrollable[1] {
			let id = ui.container_id(self);
			let area = self.area(ui);
			let mut temp: CardTemp = ui.memory_read(&id).unwrap_or(CardTemp::default());
			let scroll = Vec2::new(temp.target.x, scroll);
			temp.change(scroll, &area);
			ui.memory_save(&id, temp);
		}
	}

	/// scroll to some place, will do nothing if its not scrollable.
	pub fn scroll_delta_to(&mut self, scroll: Vec2, ui: &mut Ui) {
		self.scroll_delta_to_x(scroll.x, ui);
		self.scroll_delta_to_y(scroll.y, ui);
	}

	/// scroll to some place in x axis exactly, will do nothing if its not scrollable.
	pub fn scroll_delta_to_x(&mut self, scroll: f32, ui: &mut Ui) {
		let scroll = - scroll;
		if self.scrollable[0] {
			let id = ui.container_id(self);
			let area = self.area(ui);
			let mut temp: CardTemp = ui.memory_read(&id).unwrap_or(CardTemp::default());
			let scroll = Vec2::new(scroll + temp.target.x, temp.target.y);
			temp.change(scroll, &area);
			ui.memory_save(&id, temp);
		}
	}

	/// scroll to some place in y axis exactly, will do nothing if its not scrollable.
	pub fn scroll_delta_to_y(&mut self, scroll: f32, ui: &mut Ui) {
		let scroll = - scroll;
		if self.scrollable[1] {
			let id = ui.container_id(self);
			let area = self.area(ui);
			let mut temp: CardTemp = ui.memory_read(&id).unwrap_or(CardTemp::default());
			let scroll = Vec2::new(temp.target.x, scroll + temp.target.y);
			temp.change(scroll, &area);
			ui.memory_save(&id, temp);
		}
	}

	/// get area of [`Card`]
	pub fn get_area(&self, ui: &mut Ui) -> Area {
		let position = if self.dragable {
			let id = ui.container_id(self);
			let mut temp: CardTemp = ui.memory_read(&id).unwrap_or(CardTemp::default());
			let position = if let Some(t) = temp.position {
				t
			}else {
				temp.position = Some(self.position.unwrap_or_else(|| ui.available_position() - ui.start_position()) + ui.start_position());
				self.position.unwrap_or_else(|| ui.available_position() - ui.start_position()) + ui.start_position()
			};
			ui.memory_save(&id, temp);
			position
		}else {
			self.position.unwrap_or_else(|| ui.available_position() - ui.start_position()) + ui.start_position()
		};
		let width_and_height = if self.resizable {
			let id = ui.container_id(self);
			let mut temp: CardTemp = ui.memory_read(&id).unwrap_or(CardTemp::default());
			let size = if let Some(t) = temp.size {
				t
			}else {
				temp.size = Some(Vec2::new(
					self.width.unwrap_or_else(|| ui.window_area().right_top().x - position.x - ui.style().space),
					self.height.unwrap_or_else(|| ui.window_area().height() - position.y + ui.window_area().left_top().y - ui.style().space)
				));
				temp.size.unwrap()
			};
			ui.memory_save(&id, temp);
			size
		}else {
			Vec2::new(
				self.width.unwrap_or_else(|| ui.window_area().right_top().x - position.x - ui.style().space),
				self.height.unwrap_or_else(|| ui.window_area().height() - position.y + ui.window_area().left_top().y - ui.style().space)
			)
		};
		Area::new(position, position + width_and_height)
	}
}