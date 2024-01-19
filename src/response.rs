use crate::parse_json;
use nablo_shape::shape::shape_elements::Layer;
use crate::Key;
use crate::ClickInfo;
use crate::HoverInfo;
use crate::Instant;
use crate::MouseButton;
use crate::DragInfo;
use nablo_shape::math::Area;
use crate::InputState;
use crate::Response;
use crate::Metadata;
use time::Duration;
use nablo_shape::math::Vec2;

impl Response {
	/// is this widget was pressed this frame?
	pub fn is_pressed(&self) -> bool {
		self.metadata.click_info.pressed_mouse.len() >= 1
	}

	/// is this widget was clicked this frame?
	pub fn is_clicked(&self) -> bool {
		self.metadata.click_info.released_mouse.len() >= 1
	}

	/// is this widget was multi clicked this frame?
	pub fn is_multi_clicked(&self, multi: usize) -> bool {
		if multi > self.metadata.click_info.release_time.len() {
			return false
		}
		let mut result = true;
		for click in self.metadata.click_info.release_time.len() - multi..self.metadata.click_info.release_time.len()-1 {
			if self.metadata.click_info.release_time[click + 1] - self.metadata.click_info.release_time[click] > Duration::seconds(1) {
				result = false
			}
		}

		result
	}

	/// where do this widget clicked? [`Option::None`] for haven't clicked yet. 
	pub fn last_click_position(&self) -> Option<Vec2> {
		self.metadata.click_info.last_click_position
	}

	/// is this widget draging in this frame?
	pub fn is_draging(&self) -> bool {
		self.metadata.drag_info.is_draging
	}

	/// how much do we drag so far? [`Option::None`] for not draging
	pub fn drag(&self) -> Option<Vec2> {
		if self.metadata.drag_info.is_draging {
			return Some(self.metadata.pointer_position? - self.metadata.drag_info.last_drag_start_position?);
		}
		None
	}

	/// how long do we drag so far? [`Option::None`] for not draging
	pub fn drag_time(&self) -> Option<Duration> {
		Some(self.metadata.drag_info.drag_start_time?.elapsed())
	}

	// /// how much do we drag so far relative to lat frame?
	// pub fn drag_delta(&self) -> Option<Vec2> {
	// 	if self.metadata.drag_info.is_draging {
	// 		return Some(self.metadata.pointer_position? - self.metadata.drag_info.last_drag_position?);
	// 	}
	// 	None
	// }

	/// is this widget hovering in this frame?
	pub fn is_hovering(&self) -> bool {
		self.metadata.hover_info.is_hovering
	}

	/// is this widget lost hovering in this frame?
	pub fn is_lost_hover(&self) -> bool {
		self.metadata.hover_info.last_lost_hover_time.is_some()
	}

	/// how long do we hover so far? [`Option::None`] for not hovering
	pub fn hovering_time(&self) -> Option<Duration> {
		Some(self.metadata.hover_info.last_hover_time?.elapsed())
	}

	/// how long do we lost hover so far? [`Option::None`] for hovering
	pub fn lost_hovering_time(&self) -> Option<Duration> {
		Some(self.metadata.hover_info.last_lost_hover_time?.elapsed())
	}

	/// its a good idea to show someting, so where is the pointer? [`Option::None`] for pointer is not inside the window.
	pub fn pointer_position(&self) -> Option<Vec2> {
		self.metadata.pointer_position
	}

	/// how long do we created this widget?
	pub fn create_time(&self) -> Duration {
		self.metadata.create_time.elapsed()
	}

	/// how long do last hover sustain? [`Option::None`] for not hovered or is hovering, always be positive
	pub fn hovering_sustain_time(&self) -> Option<Duration> {
		let time = self.metadata.hover_info.last_lost_hover_time? - self.metadata.hover_info.last_hover_time?;
		if time.is_positive() {
			return Some(time);
		}
		None
	}

	/// how long since our last press? [`Option::None`] for not pressed
	pub fn press_time(&self) -> Option<Duration> {
		if self.metadata.click_info.press_time.len() == 0 {
			None
		}else {
			Some(self.metadata.click_info.press_time[self.metadata.click_info.press_time.len() - 1].elapsed())
		}
	}

	/// how long since our last release? [`Option::None`] for not releases or is pressing
	pub fn release_time(&self) -> Option<Duration> {
		if self.metadata.click_info.release_time.len() == 0 {
			None
		}else {
			Some(self.metadata.click_info.release_time[self.metadata.click_info.release_time.len() - 1].elapsed())
		}
	}

	/// how long do last press sustain? [`Option::None`] for not pressed or is pressing
	pub fn press_sustain_time(&self) -> Option<Duration> {
		let time = self.release_time()? - self.press_time()?;
		if time.is_positive() {
			return Some(time)
		}
		return None
	}

	/// get press times
	pub fn press_times(&self) -> Vec<Duration> {
		let mut back = vec!();
		for time in &self.metadata.click_info.press_time {
			back.push(time.elapsed())
		}
		back
	}

	/// get release times
	pub fn release_times(&self) -> Vec<Duration> {
		let mut back = vec!();
		for time in &self.metadata.click_info.release_time {
			back.push(time.elapsed())
		}
		back
	}

	/// read something inside memory
	pub fn memory_read<T: for<'a> serde::Deserialize<'a> + Default>(&self) -> Option<T> {
		if self.metadata.other_info.is_empty() {
			return None
		}
		parse_json(&self.metadata.other_info)
	}

	pub(crate) fn read(&mut self, metadata: &Metadata) {
		self.metadata = metadata.clone();
	}
}

impl DragInfo {
	pub(crate) fn update(&mut self, input_state: &mut InputState, area: &Area) {
		if let Some(t) = input_state.cursor_position() {
			if area.is_point_inside(&t) {
				if input_state.is_any_mouse_pressed() && !self.is_draging {
					for inner in &mut input_state.pressed_mouse {
						if !inner.is_drag_used {
							self.drag_start_time = Some(Instant::now());
							self.last_drag_start_position = Some(t);
							self.is_draging = true;
							inner.is_drag_used = true;
						}
					}
				}
			}

			if self.is_draging {
				if input_state.is_any_mouse_released() {
					self.is_draging = false;
					self.last_drag_start_position = None;
					self.drag_start_time = None;
				}
			}

			if let None = input_state.cursor_position() {
				self.is_draging = false;
				self.last_drag_start_position = None;
				self.drag_start_time = None;
			}
		}
	}
}

impl ClickInfo {
	pub(crate) fn update(&mut self, input_state: &mut InputState, area: &Area) {
		if let Some(t) = input_state.cursor_position() {
			if area.is_point_inside(&t) {
				self.pressed_mouse = input_state.pressed_mouse();
				self.released_mouse = input_state.released_mouse();
				if input_state.is_any_mouse_pressed_unconsumed() {
					self.press_time.push(Instant::now());
					input_state.consume_all_mouse_press();
				}
				if input_state.is_any_mouse_released_unconsumed() {
					self.release_time.push(Instant::now());
					input_state.consume_all_mouse_release();
				}
			}else {
				self.pressed_mouse = vec!();
				self.released_mouse = vec!();
			}
		}

		// TODO: Make this value changable
		if self.press_time.len() > 5 {
			self.press_time.remove(0);
		}

		if self.release_time.len() > 5 {
			self.release_time.remove(0);
		}
	}
}

impl HoverInfo {
	pub(crate) fn update(&mut self, input_state: &mut InputState, area: &Area) {
		if let Some(t) = input_state.cursor_position() {
			if area.is_point_inside(&t) {
				self.is_hovering = true;
				self.last_lost_hover_time = None;
				if !self.is_insert {
					self.last_hover_time = Some(Instant::now())
				}
				self.is_insert = true;
			}else {
				self.is_hovering = false;
				self.is_insert = false;
				if let None = self.last_lost_hover_time {
					self.last_lost_hover_time = Some(Instant::now())
				}
			}
		}else {
			self.is_hovering = false;
			self.is_insert = false;
			if let None = self.last_lost_hover_time {
				self.last_lost_hover_time = Some(Instant::now())
			}
		}
	}
}

impl Metadata {
	pub(crate) fn update(&mut self, input_state: &mut InputState, area: &Area) {
		self.drag_info.update(input_state, area);
		self.click_info.update(input_state, area);
		self.hover_info.update(input_state, area);
		self.pointer_position = input_state.cursor_position();
	}

	pub(crate) fn new(layer: Layer) -> Self {
		Self {
			layer,
			..Default::default()
		}
	}
}

impl Default for Metadata {
	fn default() -> Self {
		Self {
			layer: Layer::Middle,
			create_time: Instant::now(),
			hover_info: HoverInfo::default(),
			click_info: ClickInfo::default(),
			drag_info: DragInfo::default(),
			pointer_position: None,
			other_info: String::new(),
		}
	}
}

impl InputState {
	pub(crate) fn clear(&mut self) {
		self.key_repeat.clear();
		self.pressed_mouse.clear();
		self.released_key.clear();
		self.released_mouse.clear();
		self.current_scroll = Vec2::ZERO;
		self.input_text = String::new();
	}

	/// is given key pressing in this frame?
	pub fn is_key_pressing(&self, key: Key) -> bool {
		self.key.contains(&key.clone())
	}

	/// is given key pressing and pressed for some time?
	pub fn is_key_repeat(&self, key: Key) -> bool {
		self.key_repeat.contains(&key.clone())
	}

	/// is given key pressing in this frame?
	pub fn is_key_released(&self, key: Key) -> bool {
		self.released_key.contains(&key.clone())
	}

	/// is given mouse button pressed in this frame?
	pub fn is_mouse_pressed(&self, key: MouseButton) -> bool {
		for inner in &self.pressed_mouse {
			if inner.button == key {
				return true
			}
		}
		false
	}

	/// is given mouse button pressed in this frame and not consumed by other [`crate::Widget`]?
	pub fn is_mouse_pressed_unconsumed(&self, key: MouseButton) -> bool {
		for inner in &self.pressed_mouse {
			if !inner.is_click_used && inner.button == key {
				return true
			}
		}
		false
	}

	/// is any mouse click in this frame?
	pub fn is_any_mouse_pressed(&self) -> bool {
		self.pressed_mouse.len() >= 1
	}

	/// is any mouse click in this frame?
	pub fn is_any_mouse_pressed_unconsumed(&self) -> bool {
		let mut have = false;
		for inner in &self.pressed_mouse {
			if !inner.is_click_used {
				have = true
			}
		}
		have
	}

	/// is given mouse button pressing in this frame?
	pub fn is_mouse_pressing(&self, key: MouseButton) -> bool {
		self.pressing_mouse.contains(&(key.clone(), true)) || self.pressing_mouse.contains(&(key, false))
	}

	/// is given mouse button pressing in this frame and not consumed by other [`crate::Widget`]?
	pub fn is_mouse_pressing_unconsumed(&self, key: MouseButton) -> bool {
		self.pressing_mouse.contains(&(key, false))
	}

	/// is any mouse pressing in this frame?
	pub fn is_any_mouse_pressing(&self) -> bool {
		self.pressing_mouse.len() >= 1
	}

	/// is any mouse pressing in this frame?
	pub fn is_any_mouse_pressing_unconsumed(&self) -> bool {
		let mut have = false;
		for (_, is_consumed) in &self.pressing_mouse {
			if !is_consumed {
				have = true
			}
		}
		have
	}

	/// is any mouse released in this frame and not consumed by other [`crate::Widget`]?
	pub fn is_any_mouse_released_unconsumed(&self) -> bool {
		let mut have = false;
		for (_, is_consumed) in &self.released_mouse {
			if !is_consumed {
				have = true
			}
		}
		have
	}

	/// is given mouse button pressed in this frame?
	pub fn is_mouse_released(&self, key: MouseButton) -> bool {
		self.released_mouse.contains(&(key.clone(), true)) || self.released_mouse.contains(&(key, false))
	}

	/// is given mouse button pressed in this frame and not consumed by other [`crate::Widget`]?
	pub fn is_mouse_released_unconsumed(&self, key: MouseButton) -> bool {
		self.released_mouse.contains(&(key, false))
	}

	/// consume the given mouse button release event.
	pub fn consume_mouse_release(&mut self, key: MouseButton) {
		for (key_in, is_consumed) in &mut self.released_mouse {
			if *key_in == key {
				*is_consumed = true
			}
		}
	}

	/// consume the all mouse button release event.
	pub fn consume_all_mouse_release(&mut self) {
		for (_, is_consumed) in &mut self.released_mouse {
			*is_consumed = true
		}
	}

	/// consume the given mouse button press event.
	pub fn consume_mouse_press(&mut self, key: MouseButton) {
		for inner in &mut self.pressed_mouse {
			if inner.button == key {
				inner.is_click_used = true
			}
		}
	}

	/// consume the all mouse button press event.
	pub fn consume_all_mouse_press(&mut self) {
		for inner in &mut self.pressed_mouse {
			inner.is_click_used = true
		}
	}

	/// consume the given mouse button pressing event.
	pub fn consume_mouse_pressing(&mut self, key: MouseButton) {
		for (key_in, is_consumed) in &mut self.pressing_mouse {
			if *key_in == key {
				*is_consumed = true
			}
		}
	}

	/// consume the all mouse button pressing event.
	pub fn consume_all_mouse_pressing(&mut self) {
		for (_, is_consumed) in &mut self.pressing_mouse {
			*is_consumed = true
		}
	}

	/// is any mouse click in this frame?
	pub fn is_any_mouse_released(&self) -> bool {
		self.released_mouse.len() >= 1
	}

	/// what mouse button is pressed in this frame? 
	pub fn pressed_mouse(&self) -> Vec<MouseButton> {
		let mut back = vec!();
		for inner in &self.pressed_mouse {
			back.push(inner.button.clone())
		}
		back
	}

	/// what mouse button is pressing in this frame? 
	pub fn pressing_mouse(&self) -> Vec<MouseButton> {
		let mut back = vec!();
		for (key, _) in &self.pressing_mouse {
			back.push(key.clone())
		}
		back
	}

	/// what mouse button is released in this frame? 
	pub fn released_mouse(&self) -> Vec<MouseButton> {
		let mut back = vec!();
		for (key, _) in &self.released_mouse {
			back.push(key.clone())
		}
		back
	}

	/// where is cursor? for touch devices or cursor is out of window this would be [`Option::None`]
	pub fn cursor_position(&self) -> Option<Vec2> {
		self.cursor_position
	}

	/// get what txt has been input this frame
	pub fn input_text(&self) -> &String {
		&self.input_text
	}

	/// get scroll delta this frame
	pub fn scroll(&self) -> Vec2 {
		self.current_scroll
	}
}