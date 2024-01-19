use crate::OutputEvent;
use crate::PASSWORD;
use crate::widgets::TextSetting;
use nablo_shape::prelude::shape_elements::Color;
use time::Duration;
use nablo_shape::prelude::animation::Animation;
use crate::Instant;
use crate::Key;
use crate::Ui;
use crate::Response;
use crate::Widget;
use nablo_shape::prelude::Area;
use nablo_shape::prelude::Vec2;
use nablo_shape::prelude::Painter;
use crate::prelude::SingleTextInput;

impl<'a> SingleTextInput<'a> {
	/// create a new text input
	pub fn new(input: &'a mut String) -> SingleTextInput<'a> {
		Self {
			text: input.clone().into(),
			input,
			painter: Painter::default(),
			width: None,
			place_holder: String::new(),
			is_password: false,
			space: None,
			limit: None,
		}
	}

	/// set icon for a input
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

	/// check if current input have icon
	pub fn has_icon(&self) -> bool {
		!self.painter.is_empty()
	}

	/// set width of text input, by default, it will take all rest place.
	pub fn set_width(self, width: f32) -> Self {
		Self {
			width: Some(width),
			..self
		}
	}

	/// set place holder of text input.
	pub fn place_holder(self, place_holder: impl Into<String>) -> Self {
		Self {
			place_holder: place_holder.into(),
			..self
		}
	}

	/// is this input will display as password?
	pub fn password(self, is_password: bool) -> Self {
		Self {
			is_password,
			..self
		}
	}

	/// set padding of each element
	pub fn set_padding(self, padding: f32) -> Self {
		Self {
			space: Some(padding),
			..self
		}
	}

	/// set maxium input texts, by default, there's no limitation
	pub fn limit(self, limit: usize) -> Self {
		Self {
			limit: Some(limit),
			..self
		}
	}
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
struct SingleTextInputTemp {
	is_focused: bool,
	change_time: Vec<Instant>,
	pointer: usize,
	select: Option<Select>,
}

#[derive(Default, serde::Deserialize, serde::Serialize, Debug)]
struct Select {
	begin: usize,
	end: usize,
	is_backwards: bool,
}

// completely mess

impl Widget for SingleTextInput<'_> {
	fn draw(&mut self, ui: &mut Ui, response: &Response, painter: &mut Painter) {
		self.painter.change_layer(painter.style().layer);
		let background_color = ui.style().background_color.brighter(0.15);
		let space = self.space.unwrap_or(ui.style().space);
		let icon_area = self.painter.paint_area;
		let text_start = icon_area.width() + space;
		// keep for adding clear buttuon in future
		// TODO: clear and view password feature
		let space_minus = 2.0 * space;
		// text editing...
		let mut temp: SingleTextInputTemp = match response.memory_read() {
			Some(t) => t,
			None => {
				ui.memory_save(&response.id, SingleTextInputTemp::default());
				SingleTextInputTemp::default()
			}
		};
		// gain foucus
		if response.is_clicked() {
			if !temp.is_focused {
				temp.change_time.push(Instant::now());
			}
			temp.is_focused = true;
		}else if (ui.input().is_any_mouse_released() && !ui.input().cursor_position().unwrap_or(Vec2::INF).is_inside(response.area)) || ui.input().is_key_released(Key::Enter) || ui.input().is_key_released(Key::Tab) {
			if temp.is_focused {
				temp.change_time.push(Instant::now());
			}
			temp.is_focused = false;
		}
		if temp.change_time.len() > 2 {
			temp.change_time.remove(0);
		};
		if temp.is_focused {
			// insert
			let mut insert = || {
				let len = self.input.len();
				let limit = self.limit.unwrap_or(usize::MAX);
				let input_text = ui.input().input_text();
				if let Some(select) = &temp.select {
					if !input_text.is_empty() {
						*self.input = utf8_slice::till(self.input, select.begin).to_owned() + utf8_slice::from(self.input, select.end);
						temp.pointer = select.begin;
						temp.select = None;
					}
				}
				let input_text = if utf8_slice::len(input_text) >= limit.checked_sub(len).unwrap_or(0) {
					utf8_slice::till(input_text, limit.checked_sub(len).unwrap_or(0))
				}else {
					input_text
				};
				let front = utf8_slice::till(self.input, temp.pointer);
				let back = utf8_slice::from(self.input, temp.pointer);
				*self.input = front.to_owned() + input_text + back;
				temp.pointer = temp.pointer + utf8_slice::len(input_text);
			};
			let input = ui.input().clone();
			if (input.is_key_released(Key::ControlLeft) && input.is_key_released(Key::V)) |
			(input.is_key_pressing(Key::ControlLeft) && input.is_key_released(Key::V)) {
				insert()
			}else if !(input.is_key_pressing(Key::ControlLeft) || input.is_key_pressing(Key::AltLeft) || input.is_key_pressing(Key::AltRight) || input.is_key_pressing(Key::ControlRight)) {
				insert()
			}
			if (input.is_key_released(Key::ControlLeft) && input.is_key_released(Key::A)) |
			(input.is_key_pressing(Key::ControlLeft) && input.is_key_released(Key::A)) {
				temp.select = Some(Select {
					begin: 0,
					end: utf8_slice::len(self.input),
					is_backwards: false
				});
				temp.pointer = utf8_slice::len(self.input);
			}
			if (input.is_key_released(Key::ControlLeft) && input.is_key_released(Key::X)) |
			(input.is_key_pressing(Key::ControlLeft) && input.is_key_released(Key::X)) {
				if let Some(select) = &temp.select {
					ui.send_output_event(OutputEvent::ClipboardCopy(utf8_slice::slice(self.input, select.begin,select.end).to_string()));
					temp.pointer = select.begin;
					*self.input = utf8_slice::till(self.input, select.begin).to_owned() + utf8_slice::from(self.input, select.end);
					temp.select = None;
				}
			};
			if (input.is_key_released(Key::ControlLeft) && input.is_key_released(Key::C)) |
			(input.is_key_pressing(Key::ControlLeft) && input.is_key_released(Key::C)) {
				if let Some(select) = &temp.select {
					ui.send_output_event(OutputEvent::ClipboardCopy(utf8_slice::slice(self.input, select.begin,select.end).to_string()));
				}
			};

			// delete
			if ui.input().is_key_repeat(Key::Backspace) {
				if let Some(select) = &temp.select {
					*self.input = utf8_slice::till(self.input, select.begin).to_owned() + utf8_slice::from(self.input, select.end);
					temp.pointer = select.begin;
					temp.select = None;
				}else {
					if temp.pointer != 0 {
						let front = utf8_slice::till(self.input, temp.pointer - 1);
						let back = utf8_slice::from(self.input, temp.pointer);
						*self.input = front.to_owned() + back;
						temp.pointer = temp.pointer - 1
					}
				}
			}

			// move pointer and select
			if ui.input().is_key_repeat(Key::ArrowLeft) {
				if ui.input().is_key_pressing(Key::ShiftLeft) {
					if let Some(t) = &mut temp.select {
						if t.is_backwards {
							t.begin = t.begin.checked_sub(1).unwrap_or(0);
						}else {
							t.end = t.end.checked_sub(1).unwrap_or(0);
							if t.end == t.begin {
								t.is_backwards = true
							}
						}
					}else {
						if temp.pointer != 0 {
							temp.select = Some(Select {
								begin: temp.pointer - 1,
								end: temp.pointer,
								is_backwards: true,
							});
						}
					}
					temp.pointer = temp.pointer.checked_sub(1).unwrap_or(0);
				}else {
					if temp.pointer != 0 {
						temp.pointer = temp.pointer - 1;
					}
					temp.select = None;
				}
			}else if ui.input().is_key_repeat(Key::ArrowRight) {
				if ui.input().is_key_pressing(Key::ShiftLeft) {
					if let Some(t) = &mut temp.select {
						let add_with_ceil = |input: &mut usize| { 
							if input < &mut utf8_slice::len(self.input) { 
								*input = *input + 1; 
							} 
						}; 
						if t.is_backwards {
							add_with_ceil(&mut t.begin);
							if t.begin == t.end {
								t.is_backwards = false
							}
						}else {
							add_with_ceil(&mut t.end);
						}
					}else {
						if temp.pointer < utf8_slice::len(self.input) {
							temp.select =  Some(Select {
								begin: temp.pointer,
								end: temp.pointer + 1,
								is_backwards: false,
							});
						}
					}
					if temp.pointer < utf8_slice::len(self.input) {
						temp.pointer = temp.pointer + 1;
					}
				}else {
					if temp.pointer < utf8_slice::len(self.input) {
						temp.pointer = temp.pointer + 1;
					}
					temp.select = None;
				}
			}
			if response.is_clicked() {
				fn compress(input: f32) -> f32 {
					if input > 1.0 {
						return 1.0;
					}else if input < 0.0 {
						return 0.0;
					}
					input
				}
				let cursor_position = ui.input().cursor_position().unwrap_or(Vec2::INF);
				let text_width = painter.text_area(self.input.to_string()).width();
				let len = if response.area.width() < text_width {
					1.0 - (response.area.area[1].x - cursor_position.x) / text_width
				}else {
					(cursor_position.x - response.area.area[0].x - text_start) / text_width
				}; 
				let len = compress(len);
				let len = (len * utf8_slice::len(self.input) as f32) as usize;
				temp.pointer = len;
				temp.select = None;
			}
			if self.input.is_empty() {
				temp.select = None
			}
		}
		ui.memory_save(&response.id, &temp);

		// animation caculate
		let animation_time = Duration::milliseconds(250);
		let animation = Animation::new_standard(animation_time, Vec2::new(0.3, 0.0), Vec2::new(0.7, 1.0));
		let brighter = if temp.change_time.len() == 2 {
			let delta = temp.change_time[0].elapsed() - temp.change_time[1].elapsed();
			let calc = if delta > animation_time {
				animation.caculate(&temp.change_time[1].elapsed()).unwrap_or_else(|| 1.0)
			}else {
				animation.caculate(&(delta + temp.change_time[1].elapsed())).unwrap_or_else(|| 1.0)
			};
			if temp.is_focused {
				calc
			}else {
				1.0 - calc
			}
		}else if temp.change_time.len() == 1 {
			let calc = animation.caculate(&temp.change_time[0].elapsed()).unwrap_or_else(|| 1.0);
			if temp.is_focused {
				calc
			}else {
				1.0 - calc
			}
		}else {
			0.0
		};
		let brighter = brighter * 0.5 + 0.5;
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

		// draw
		// # background
		if self.is_password {
			self.text.text = (0..utf8_slice::len(self.input)).into_iter().map(|_| PASSWORD).collect();
		}else {
			self.text.text = self.input.clone();
		}
		painter.set_color(background_color);
		painter.set_position(response.area.area[0]);
		let stroke_color: Color = 1.0.into();
		let stroke_color = stroke_color.set_alpha((brighter * 255.0) as u8);
		painter.set_stroke_color(stroke_color);
		painter.set_stroke_width(1.0);
		painter.rect(response.area.width_and_height(), Vec2::same(2.5));
		// # icon
		let position = response.area.area[0] + Vec2::new(space / 2.0, (response.area.height() - icon_area.height()) / 2.0);
		self.painter.move_delta_to(position);
		self.painter.change_clip(painter.style().clip);
		painter.append(&mut self.painter);
		// # text
		if self.text.text.is_empty() {
			self.text.text = self.place_holder.clone();
			self.text.color = Some([0.5,0.5,0.5,0.5].into());
		}
		let front = utf8_slice::till(&self.text.text, temp.pointer).to_string();
		let x = if painter.text_area(front.clone()).width() < response.area.width() - space_minus {
			text_start
		}else {
			text_start + response.area.width() - space_minus - painter.text_area(front.clone()).width()
		};
		let y = (response.area.height() - 16.0) / 2.0;
		let position = response.area.area[0] + Vec2::new(x, y);
		painter.set_clip([text_start + response.area.area[0].x, response.area.area[0].y, response.area.area[1].x - space, response.area.area[1].y].into());
		self.text = self.text.clone().set_width(response.area.width());
		self.text.text_draw(painter, position, ui);
		// # pointer
		let pointer = painter.text_area(front).width() + x;
		let position = response.area.area[0] + Vec2::new(pointer, (response.area.height() - 16.0) / 2.0);
		painter.set_clip([text_start + response.area.area[0].x, response.area.area[0].y, response.area.area[1].x, response.area.area[1].y].into());
		painter.set_color([1.0,1.0,1.0,brighter * 2.0 - 1.0]);
		painter.set_position(position);
		painter.set_stroke_width(0.0);
		painter.rect([2.0, 16.0].into(), Vec2::ZERO);
		// # select
		if let Some(select) = temp.select {
			let text = utf8_slice::till(&self.text.text, select.begin).to_string();
			let front_x = painter.text_area(text).width() + x;
			let text = utf8_slice::slice(&self.text.text, select.begin, select.end).to_string();
			let select_width = painter.text_area(text).width();
			let position = response.area.area[0] + Vec2::new(front_x, y);
			painter.set_color(ui.style().primary_color.set_alpha(100));
			painter.set_position(position);
			painter.rect([select_width, 16.0].into(), Vec2::ZERO);
			// println!("{:?} front_x: {}, select_width: {}", select, front_x, select_width);
		}

		painter.brighter(light_factor);
	}

	fn ui(&mut self, ui: &mut Ui, area: Option<Area>) -> Response {
		let space = self.space.unwrap_or(ui.style().space);
		let height = if self.has_icon() {
			let icon_area = self.painter.paint_area;
			icon_area.height() + space
		}else {
			space * 2.0
		};
		let width = match self.width {
			Some(t) => t,
			None => ui.window_area().right_top().x - ui.available_position().x - space
		};
		let area = match area {
			Some(t) => t,
			None => Area::new(ui.available_position(), ui.available_position() + Vec2::new(width, height))
		};
		ui.response(area)
	}
}