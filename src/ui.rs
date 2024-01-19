use std::ops::RangeInclusive;
use crate::prelude::*;
use crate::PaintStyle;
use crate::Container;
use crate::InnerResponse;
use crate::Key;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use image::io::Reader;
use resvg::usvg::TreeParsing;
use crate::OutputEvent;
use crate::texture::Image;
use image::GenericImageView;
use crate::Shapes;
use crate::prelude::Empty;
use crate::MemoryTemp;
use rayon::prelude::*;
use crate::parse_json;
use crate::to_json;
use crate::Metadata;
use crate::Layout;
#[cfg(feature = "vertexs")]
use nablo_shape::shape::ShapeElement;
use nablo_shape::shape::Painter;
use time::Duration;
use crate::Event;
use crate::Response;
use crate::Widget;
use nablo_shape::math::Area;
use crate::InputState;
use crate::Ui;
use nablo_shape::math::Vec2;
use crate::Style;
use crate::Instant;
use std::collections::HashMap;
use anyhow::Result;

impl Default for Ui {
	fn default() -> Self {
		Self {
			memory: HashMap::new(),
			memory_clip: vec!(),
			memory_clip_total: vec!(),
			shape: Shapes::default(),
			available_position: Vec2::same(Style::default().space),
			last_frame: Instant::now(),
			events: vec!(),
			input_state: InputState::default(),
			window: Area::default(),
			style: Style::default(),
			available_id: (String::new(), 0),
			language: String::new(),
			paint_style: PaintStyle::default(),
			layout: Layout::default(),
			output_events: vec!(),
			texture_id: vec!(),
			offset: Vec2::ZERO,
			parent_area: None
		}
	}
}

impl Ui {
	/// add a widget
	pub fn add(&mut self, widget: impl Widget) -> &Response {
		let id = self.available_id();
		self.add_with_id(id, widget)
	}

	/// add a widget with id
	pub(crate) fn add_with_id(&mut self, id: String, mut widget: impl Widget) -> &Response {
		let response = widget.ui(self, None);
		self.add_widget(id, widget, response)
	}

	/// put a widget
	pub fn put(&mut self, widget: impl Widget, area: Area) -> &Response {
		let id = self.available_id();
		self.put_with_id(id, widget, area)
	}

	/// put a widget with id
	pub(crate) fn put_with_id(&mut self, id: String, mut widget: impl Widget, area: Area) -> &Response {
		let response = widget.ui(self, Some(area));
		self.add_widget(id, widget, response)
	}

	/// add a widget with exist response.
	pub fn add_widget(&mut self, id: String, mut widget: impl Widget, response: Response) -> &Response {
		let mut response = response;
		self.memory_clip.push(id.clone());
		self.memory_clip_total.push(id.clone());
		response.id = id.clone();
		if let Some(t) = self.memory.get_mut(&id) {
			t.access_time = t.access_time + 1;
			let area = response.area;
			response.read(&t.response.metadata);
			response.area = area;
			t.response = response.clone();
			let mut shapes = self.painter();
			widget.draw(self, &response, &mut shapes);
			self.shape.append(shapes);
		}else {
			self.memory.insert(id.clone(), MemoryTemp {
				response,
				access_time: 1
			});
		};
		&self.memory.get(&id).unwrap().response
	}

	/// to show your container
	pub fn show<R, C: Container>(&mut self, container: &mut C, inner_widget: impl FnOnce(&mut Ui, &mut C) -> R) -> InnerResponse<R> {
		let id = container.get_id(self);
		let size = container.area(self);
		let area = size.cross_part(&self.window);
		let layer = container.layer(self);
		let shapes_len = self.shape.raw_shape.len();
		let mut painter = Painter::from_area(&area);
		painter.set_layer(layer); 
		let response = self.response_update(size, id.clone()).clone();
		let if_show = container.begin(self, &mut painter, &response);
		let style = painter.style().clone();
		let offset = painter.offset;
		self.shape.append(painter);
		if if_show {
			let return_value = InnerResponse {
				response,
				..self.sub_ui(size, id, style.clone(), offset, container, inner_widget)
			};
			let split = self.shape.raw_shape.split_off(shapes_len);
			let mut painter = Painter::new(&size, split, style);
			container.end(self, &mut painter, &return_value);
			self.shape.append(painter);
			return_value
		}else {
			InnerResponse {
				response,
				inner_responses: vec!(),
				return_value: None
			}
		}
		
	}

	/// add a response area to ui
	pub fn response(&mut self, area: Area) -> Response {
		let mut area = area;
		self.position_change(&mut area);
		Response {
			area,
			metadata: Metadata::new(self.paint_style.layer),
			..Default::default()
		}
	}

	/// add a response area to ui, and add a update task before next frame
	pub fn response_update(&mut self, area: Area, id: String) -> &Response {
		let response = self.response(area);
		self.add_widget(id, Empty{} , response)
	}

	/// get where should we print
	pub fn available_position(&self) -> Vec2 {
		self.available_position
	} 

	/// get how large current window is, Note: if you're using sub_ui, this area's left top point will not be [`Vec2::ZERO`].
	pub fn window_area(&self) -> Area {
		self.window
	}

	/// the way you paint stuff in `nablo`, note: actually you cant use this to paint something...
	pub fn painter(&self) -> Painter {
		let area = if let Some(t) = self.parent_area {
			self.window.cross_part(&t)
		}else {
			self.window
		};
		let mut back = Painter::from_area(&area);
		*back.style_mut() = self.paint_style.clone();
		back
	}

	/// how long have passed since last frame?
	pub fn delay(&self) -> Duration {
		self.last_frame.elapsed()
	}

	/// get [`InputState`]
	pub fn input(&self) -> &InputState {
		&self.input_state
	}

	/// get [`InputState`], but muttable
	pub fn input_mut(&mut self) -> &mut InputState {
		&mut self.input_state
	}

	/// get style on this ui
	pub fn style(&self) -> &Style {
		&self.style
	}

	/// get mutable style on this ui
	pub fn style_mut(&mut self) -> &mut Style {
		&mut self.style
	}

	/// get style on this ui
	pub fn paint_style(&self) -> &PaintStyle {
		&self.paint_style
	}

	/// get mutable style on this ui
	pub fn paint_style_mut(&mut self) -> &mut PaintStyle {
		&mut self.paint_style
	}

	/// save someting in `nablo` memory, *dont* save your data in there. This should be designed to save all kinds of datas....
	/// but i dont find better ways.
	/// too bad!
	///
	/// note you must make sure that the id you put in is id of a added [`crate::Widget`] or [`crate::Container`], otherwise the data will not be saved.
	pub fn memory_save(&mut self, id: &String, data: impl serde::Serialize) {
		if let Some(t) = self.memory.get_mut(id) {
			t.response.metadata.other_info = to_json(&data)
		}
	}

	/// read someting in `nablo` memory, will send out default value if deserilizing error occurs. [`Option::None`] for not find value
	pub fn memory_read<T: for<'a> serde::Deserialize<'a> + Default>(&mut self, id: &String) -> Option<T> {
		let back = self.memory.get(id)?;
		parse_json(&back.response.metadata.other_info)
	}

	/// a id alloctor, changes every time this function called. may be stupid.
	pub fn available_id(&mut self) -> String {
		let (ui_id, num_id) = &mut self.available_id;
		let back = format!("{ui_id}----{num_id}");
		*num_id = *num_id + 1;
		back
	}

	/// to get current language, for multi language support
	pub fn language(&self) -> String {
		self.language.clone()
	}

	/// to change current language, for multi language support. feel free to use your way to stand for a language.
	pub fn change_language(&mut self, language: String){
		self.language = language;
	}

	/// to get a sub reigon of current window, usually used in [`crate::Container`], returns all inner element's [`crate::Response`] and a empty [`crate::Response`].
	pub fn sub_ui<R, C: Container>(&mut self, area: Area, id: String, paint_style: PaintStyle, offset: Vec2, container: &mut C ,widgets: impl FnOnce(&mut Ui, &mut C) -> R) -> InnerResponse<R> {
		let mut sub_ui = Self {
			memory: self.memory.clone(),
			memory_clip: self.memory_clip.clone(),
			memory_clip_total: self.memory_clip_total.clone(),
			available_position: area.left_top() + Vec2::same(self.style.space) + offset,
			window: self.window.cross_part(&area),
			style: self.style.clone(),
			last_frame: self.last_frame,
			events: self.events.clone(),
			input_state: self.input_state.clone(),
			available_id: (id.clone(), 0),
			paint_style,
			language: self.language.clone(),
			texture_id: self.texture_id.clone(),
			offset,
			parent_area: Some(self.window_area()), 
			..Default::default()
		};
		let return_value = widgets(&mut sub_ui, container);
		let inner_response: Vec<Response> = sub_ui.memory.clone().into_par_iter().filter_map(|(key, response)| {
			let binding: Vec<&str> = key.split("----").collect();
			if binding.len() == 2 {
				if binding[0] == id {
					Some(response.response.clone())
				}else {
					None
				}
			}else {
				None
			}
		}).collect();
		let area = Area::ZERO;
		let cover = Response {
			area,
			metadata: Metadata::new(self.paint_style.layer),
			..Default::default()
		};
		self.memory = sub_ui.memory;
		self.input_state = sub_ui.input_state;
		self.style = sub_ui.style;
		self.shape.raw_shape.append(&mut sub_ui.shape.raw_shape);
		self.output_events.append(&mut sub_ui.output_events);
		self.texture_id = sub_ui.texture_id;
		self.memory_clip = sub_ui.memory_clip;
		self.memory_clip_total = sub_ui.memory_clip_total;
		(cover, inner_response, return_value).into()
	}

	/// returns events
	pub fn events(&mut self) -> &Vec<Event> {
		&self.events
	}

	/// send a output event to host
	pub fn send_output_event(&mut self, output_event: OutputEvent) {
		self.output_events.push(output_event);
	}

	pub(crate) fn position_change(&mut self, area: &mut Area) {
		if self.layout.is_inverse {
			if self.layout.is_horizental {
				self.available_position	= self.available_position - Vec2::new(area.width() + self.style.space, 0.0);
				area.move_delta_to(Vec2::new(-area.width(), 0.0));
			}else {
				self.available_position	= self.available_position - Vec2::new(0.0, area.height() + self.style.space);
				area.move_delta_to(Vec2::new(0.0, -area.height()));
			}
		}else {
			if self.layout.is_horizental {
				self.available_position	= self.available_position + Vec2::new(area.width() + self.style.space, 0.0);
			}else {
				self.available_position	= self.available_position + Vec2::new(0.0, area.height() + self.style.space);
			}
		}
	}

	/// this function will collect added widgets between last called, will automaticly called during every loop.
	pub(crate) fn count(&mut self) -> Vec<&Response> {
		let back = self.memory_clip.clone().into_iter().filter_map(|x| {
			Some(&self.memory.get(&x)?.response)
		}).collect();
		self.memory_clip.clear();
		back
	}

	pub(crate) fn event(&mut self, event: &Event) {
		match event {
			Event::KeyPressed(key) => {
				if !self.input_state.key.contains(key) {
					self.input_state.key.push(key.clone());
				}
				self.input_state.key_repeat.push(key.clone());
				if !(self.input_state.is_ime_on 
				|| self.input_state.is_key_pressing(Key::ControlLeft) 
				|| self.input_state.is_key_pressing(Key::ControlLeft)
				|| self.input_state.is_key_pressing(Key::Tab) 
				|| self.input_state.is_key_pressing(Key::AltLeft)
				|| self.input_state.is_key_pressing(Key::AltRight)
				){
					self.input_state.input_text = self.input_state.input_text.clone() + &key.to_string(self.input().is_key_pressing(Key::ShiftLeft) || self.input().is_key_pressing(Key::ShiftRight));
				}
			},
			Event::KeyRelease(key) => {
				self.input_state.key.retain(|btn| btn != key);
				self.input_state.released_key.push(key.clone());
			},
			Event::CursorMoved(position) => self.input_state.cursor_position = Some(*position),
			Event::CursorEntered => self.input_state.cursor_position = Some(Vec2::new(0.0,0.0)),
			Event::CursorLeft =>  self.input_state.cursor_position = None,
			Event::MouseClick(button) => {
				self.input_state.pressed_mouse.push(button.clone().into());
				self.input_state.pressing_mouse.push((button.clone(), false));
				self.input_state.released_mouse.retain(|(btn, _)| btn != button);
				self.input_state.click_time.insert(button.clone(), Instant::now());
			},
			Event::MouseRelease(button) => {
				self.input_state.pressing_mouse.retain(|(btn, _)| btn != button);
				self.input_state.released_mouse.push((button.clone(), false));
				self.input_state.click_time.remove(button);
			},
			Event::Resized(size) => {
				self.window = Area::new_with_origin(*size)
			}
			Event::TextInput(text) => {
				self.input_state.input_text = text.to_string();
			}
			Event::TouchStart(touch) => {
				self.input_state.touch.insert(touch.id, (touch.clone(), false));
			}
			Event::TouchMove(touch) => {
				self.input_state.touch.insert(touch.id, (touch.clone(), false));
			}
			Event::TouchEnd(touch) => {
				self.input_state.touch.remove(&touch.id);
			}
			Event::TouchCancel(touch) => {
				self.input_state.touch.remove(&touch.id);
			},
			Event::Scroll(scroll) => self.input_state.current_scroll = *scroll,
			Event::ImeEnable => self.input_state.is_ime_on = true,
			Event::ImeDisable => self.input_state.is_ime_on = false,
			Event::NotSupported => {},
		};
		if let Event::NotSupported =  event {}
		else {
			self.events.push(event.clone())
		}
	}

	pub(crate) fn raw_shape(&mut self) {
		self.shape.raw_shape.par_sort_by(|a, b| a.style.layer.cmp(&b.style.layer));
	}

	#[cfg(feature = "vertexs")]
	pub(crate) fn handle_raw_shape(&mut self) {
		self.raw_shape();
		let shapes = self.shape.raw_shape.clone();
		self.shape.raw_shape.clear();
		for shape in shapes {
			if let ShapeElement::Text(_) = shape.shape {
				self.shape.push_text(shape.style.layer, shape.clone())
			}else if let ShapeElement::Image(_) = shape.shape {
				self.shape.push_image(shape.style.layer, shape.clone())
			}else {
				self.shape.push_shape(shape.style.layer, shape.into_vertexs(self.window.width_and_height(), self.shape.vertexs_len()))
			}
		}
	}

	pub(crate) fn clear(&mut self) {
		self.shape.clear();
		self.input_state.clear();
		self.events.clear();
		self.available_position = Vec2::same(self.style.space);
		self.last_frame = Instant::now();
		self.available_id.1 = 0;
		self.memory_clip.clear();
		self.output_events.clear();
		// for borrow checker
		let mut remove_key = vec!();
		for (key, temp) in &mut self.memory {
			if temp.access_time > 1 {
				println!("Warn: Id conflict: {}, access time: {}", key, temp.access_time);
			}
			if temp.access_time == 0 {
				remove_key.push(key.clone());
			}else {
				temp.access_time = 0;
			};
		}
		for key in remove_key {
			self.memory.remove(&key);
		}
	}

	pub(crate) fn update(&mut self) {
		let mut data_0 = vec!();
		let mut data_1 = vec!();
		let mut data_2 = vec!();
		let mut data_3 = vec!();
		let mut data_4 = vec!();
		let mut data_5 = vec!();
		for id in self.memory_clip_total.iter().rev() {
			if let Some(res) = self.memory.remove(id) {
				match res.response.metadata.layer {
					Layer::Background => data_5.push((id.clone(), res)),
					Layer::Bottom => data_4.push((id.clone(), res)),
					Layer::Middle => data_3.push((id.clone(), res)),
					Layer::Foreground => data_2.push((id.clone(), res)),
					Layer::ToolTips => data_1.push((id.clone(), res)),
					Layer::Debug => data_0.push((id.clone(), res))
				}
			}
		}
		let mut update = |input: &mut Vec<(String, MemoryTemp)>| {
			for (id ,res) in input {
				res.response.metadata.update(&mut self.input_state, &res.response.area);
				self.memory.insert(id.to_string(), res.clone());
			}
		};
		update(&mut data_0);
		update(&mut data_1);
		update(&mut data_2);
		update(&mut data_3);
		update(&mut data_4);
		update(&mut data_5);
		self.memory_clip_total.clear();
	}
}

impl Ui {
	/// # Layouts

	/// change the way we put widgets.
	pub fn layout<R>(&mut self, layout: Layout, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
		let original_layout = self.layout.clone();
		let original_position = self.available_position;
		self.layout = layout;
		if self.layout.is_inverse {
			if self.layout.is_horizental {
				self.available_position = Vec2::new(self.window.right_top().x - self.style.space, self.available_position.y) + self.offset;
			}else {
				self.available_position = Vec2::new(self.available_position.x, self.window.left_bottom().y - self.style.space)+ self.offset;
			}
		}
		self.count();
		let return_value = add_contents(self);
		let responses = self.count();
		let mut area = Area::ZERO;
		for res in responses {
			area.combine(&res.area)
		}
		self.available_position = original_position;
		self.layout = original_layout;
		self.position_change(&mut area);
		return_value
	}

	/// put widgets horizentally
	pub fn horizental<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
		self.layout(Layout::horizental(), add_contents)
	}

	/// put widgets horizentally and backwards
	pub fn horizental_inverse<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
		self.layout(Layout::horizental_inverse(), add_contents)
	}

	/// put widgets vertically
	pub fn vertical<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
		self.layout(Layout::vertical(), add_contents)
	}

	/// put widgets vertically and backwards
	pub fn vertical_inverse<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
		self.layout(Layout::vertical_inverse(), add_contents)
	}
}

impl Ui {
	/// # Textures

	#[cfg(not(target_arch = "wasm32"))]
	/// add a texture from path, this function will add a texture when only current id is not been taken. for svg using [`Self::create_texture_svg`]
	pub fn create_texture_from_path<P: AsRef<Path>>(&mut self, path: P, id: impl Into<String>) -> Result<()> {
		let id = id.into();
		if !self.texture_id.contains(&id) {
			let diffuse_image = Reader::open(path)?.decode()?;
			let diffuse_rgba = diffuse_image.to_rgba8();
			let dimensions = diffuse_image.dimensions();
			let image = Image {
				rgba: diffuse_rgba.to_vec(),
				size: Vec2::new(dimensions.0 as f32, dimensions.1 as f32),
				id: id.clone(),
			};
			self.output_events.push(OutputEvent::TextureChange(image));
			self.texture_id.push(id);
		}
		Ok(())
	}

	/// add a texture, this function will add a texture when only current id is not been taken. for svg using [`Self::create_texture_svg`]
	pub fn create_texture(&mut self, bytes: &[u8], id: impl Into<String>) -> Result<()> {
		let id = id.into();
		if !self.texture_id.contains(&id) {
			self.output_events.push(OutputEvent::TextureChange(texture(bytes, id.clone())?));
			self.texture_id.push(id);
		}
		Ok(())
	}

	/// add a texture with a svg file, this function will add a texture when only current id is not been taken.
	///
	/// # Panics
	/// `size.x < 0.0 || size.y < 0.0 || size.x.is_infinite() || size.y.is_infinite()`
	pub fn create_texture_svg(&mut self, bytes: &[u8], size: Vec2, id: impl Into<String>) -> Result<()> {
		let id = id.into();
		if !self.texture_id.contains(&id) {
			self.output_events.push(OutputEvent::TextureChange(texture_svg(bytes, size, id.clone())?));
			self.texture_id.push(id);
		}
		Ok(())
	}

	/// change a texture, this function will add a texture when a texture is not added. for svg using [`Self::change_texture_svg`]
	///
	/// Note: *dont* call this every frame, or `nablo` will be unstandably lagging.
	pub fn change_texture(&mut self, bytes: &[u8], id: impl Into<String>) -> Result<()> {
		let id = id.into();
		self.output_events.push(OutputEvent::TextureChange(texture(bytes, id.clone())?));
		if !self.texture_id.contains(&id) {
			self.texture_id.push(id);
		}
		Ok(())
	}

	/// change a texture with a svg file, this function will add a texture when a texture is not added.
	///
	/// Note: *dont* call this every frame, or `nablo` will be unstandably lagging.
	///
	/// # Panics
	/// `size.x < 0.0 || size.y < 0.0 || size.x.is_infinite() || size.y.is_infinite()`
	pub fn change_texture_svg(&mut self, bytes: &[u8], size: Vec2, id: impl Into<String>) -> Result<()> {
		let id = id.into();
		self.output_events.push(OutputEvent::TextureChange(texture_svg(bytes, size, id.clone())?));
		if !self.texture_id.contains(&id) {
			self.texture_id.push(id);
		}
		Ok(())
	}

	/// delete a texture by using id
	pub fn delete_texture(&mut self, id: impl Into<String>) {
		let id = id.into();
		self.texture_id.retain(|a| a != &id);
		self.output_events.push(OutputEvent::TextureDelete(id));
	}
}

impl Ui {
	/// # widgets shortcuts

	/// add a [`crate::widgets::Label`].
	pub fn label(&mut self, text: impl Into<Text>) -> &Response {
		self.add(Label::new(text))
	}

	/// add a [`crate::widgets::Button`].
	pub fn button(&mut self, text: impl Into<Text>) -> &Response {
		self.add(Button::new(text))
	}

	/// add a [`crate::widgets::Canvas`].
	pub fn canvas<P: FnOnce(&mut Painter)>(&mut self, width_and_height: Vec2, paint: P) -> &Response {
		self.add(Canvas::new(width_and_height, paint))
	}

	/// add a [`crate::widgets::DragableValue`].
	pub fn dragable_value<'a, T: Num>(&mut self, input: &'a mut T) -> &Response {
		self.add(DragableValue::new(input))
	}

	/// add a [`crate::widgets::Slider`].
	pub fn slider<'a, T: Num>(&mut self, range: RangeInclusive<T> ,input: &'a mut T) -> &Response {
		self.add(Slider::new(range, input))
	}

	/// add a [`crate::widgets::SingleTextInput`].
	pub fn single_input<'a>(&mut self, input: &'a mut String) -> &Response {
		self.add(SingleTextInput::new(input))
	}

	/// add a [`crate::widgets::SelectableValue`].
	pub fn switch(&mut self, select: &mut bool, text: impl Into<Text>) -> &Response {
		let res = self.add(SelectableValue::new(*select, text));
		if res.is_clicked() {
			*select = !*select
		}
		res
	}

	/// add a [`crate::widgets::SelectableValue`].
	pub fn selectable_value<T: Eq>(&mut self, input: &mut T, select: T, text: impl Into<Text>) -> &Response {
		let mut select = select;
		let res = self.add(SelectableValue::new(input == &mut select, text));
		if res.is_clicked() {
			*input = select
		}
		res
	}
}

impl Ui {
	/// # container shortcuts

	/// add a [`crate::container::Card`]
	pub fn card<R>(&mut self, id: impl Into<String>, width_and_height: Vec2, inner_widget: impl FnOnce(&mut Ui, &mut Card) -> R) -> InnerResponse<R> {
		self.show(&mut Card::new(id).set_width(width_and_height.x).set_height(width_and_height.y).set_scrollable([true; 2]), inner_widget)
	}

	/// add a [`crate::container::Card`] with dragable on, can be a simulate to window
	pub fn window<R>(&mut self, id: impl Into<String>, width_and_height: Vec2, inner_widget: impl FnOnce(&mut Ui, &mut Card) -> R) -> InnerResponse<R> {
		self.show(&mut Card::new(id).set_width(width_and_height.x).set_height(width_and_height.y).set_scrollable([true; 2]).set_dragable(true), inner_widget)
	}

	/// add a [`crate::container::Collapsing`]
	pub fn collapsing<R>(&mut self, id: impl Into<String>, inner_widget: impl FnOnce(&mut Ui, &mut Collapsing) -> R) -> InnerResponse<R> {
		self.show(&mut Collapsing::new(id), inner_widget)
	}

	/// add a [`crate::container::MessageProvider`]
	pub fn message_provider<R>(&mut self, id: impl Into<String>, inner_widget: impl FnOnce(&mut Ui, &mut MessageProvider) -> R) -> InnerResponse<R> {
		self.show(&mut MessageProvider::new(id), inner_widget)
	}
}

fn texture(bytes: &[u8], id: String) -> Result<Image> {
	let diffuse_image = image::load_from_memory(&bytes)?;
	let diffuse_rgba = diffuse_image.to_rgba8();
	let dimensions = diffuse_image.dimensions();
	Ok(Image {
		rgba: diffuse_rgba.to_vec(),
		size: Vec2::new(dimensions.0 as f32, dimensions.1 as f32),
		id
	})
}

fn texture_svg(bytes: &[u8], size: Vec2, id: String) -> Result<Image> {
	let tree = resvg::Tree::from_usvg(&resvg::usvg::Tree::from_data(bytes, &resvg::usvg::Options {
		default_size: resvg::usvg::Size::from_wh(size.x, size.y).expect("invaild size"),
		..Default::default()
	})?);
	let mut pixmap = resvg::tiny_skia::Pixmap::new(size.x as u32, size.y as u32).expect("invaild size");
	let mut pixmap_mut = pixmap.as_mut();
	tree.render(resvg::tiny_skia::Transform::identity(), &mut pixmap_mut);
	Ok(Image {
		rgba: pixmap_mut.to_owned().data().to_vec(),
		size,
		id
	})
}

impl Layout {
	/// # buileders
	pub fn horizental() -> Self {
		Self {
			is_inverse: false,
			is_horizental: true
		}
	}

	pub fn horizental_inverse() -> Self {
		Self {
			is_inverse: true,
			is_horizental: true
		}
	}

	/// actually this is the default way we put widgets
	pub fn vertical() -> Self {
		Self {
			is_inverse: false,
			is_horizental: false
		}
	}

	pub fn vertical_inverse() -> Self {
		Self {
			is_inverse: true,
			is_horizental: false
		}
	}
}