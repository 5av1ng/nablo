use nablo_shape::prelude::shape_elements::Color;
use rayon::prelude::*;
use crate::prelude::*;

impl MessageProvider {
	/// create a new message provider
	pub fn new(id: impl Into<String>) -> Self {
		Self {
			id: id.into()
		}
	}

	/// display a new message
	pub fn message(&self, message: impl Into<Message>, ui: &mut Ui) {
		let message = message.into();
		let id = ui.container_id(self);
		let mut temp: MessageTemp = ui.memory_read(&id).unwrap_or_default();
		if !temp.messages.contains_key(&message.id) {
			temp.messages.insert(message.id.clone(), (message, Instant::now()).into());
		}
		ui.memory_save(&id, &temp);
	}

	/// change a exist message, will do nothing if requested message doest exist
	pub fn message_change(&self, message_id: &String, change: impl FnOnce(&mut Message), ui: &mut Ui) {
		let id = ui.container_id(self);
		let mut temp: MessageTemp = ui.memory_read(&id).unwrap_or_default();
		if let Some(t) = temp.messages.get_mut(message_id) {
			change(&mut t.message);
		};
		ui.memory_save(&id, &temp);
	}
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
struct MessageTemp {
	messages: HashMap<String, MessageInner>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct MessageInner {
	message: Message,
	create_time: Instant,
	delete_time: Option<Instant>
}

impl From<(Message, Instant)> for MessageInner {
	fn from(val: (Message, Instant)) -> Self {
		let (message, create_time) = val;
		MessageInner {
			message,
			create_time,
			delete_time: None
		}
	}
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
/// a struct represents a message
pub struct Message {
	/// the message itself
	pub text: Text,
	/// how long should we sustain, [`Option::None`] for display infinatly until you close it manually.
	pub sustain_time: Option<Duration>,
	/// set icon to a message
	pub icon: Painter,
	/// the id of a message
	pub id: String,
	/// if we should delete this message now?
	pub should_delete: bool,
	/// current status of the message
	pub status: Status,
}

impl Message {
	pub fn status(self, status: Status) -> Self {
		Self {
			status,
			..self
		}
	}
}

impl Default for Message {
	fn default() -> Self {
		Self {
			text: "".into(),
			sustain_time: Some(Duration::seconds(3)),
			icon: Painter::default(),
			id: "".into(),
			should_delete: false,
			status: Status::Default
		}
	}
}

impl<T> From<T> for Message where
	T: Into<String>
{
	fn from(value: T) -> Self {
		let value = value.into();
		Self {
			id: value.clone(),
			text: value.into(),
			..Default::default()
		}
	}
}

impl Container for MessageProvider {
	fn get_id(&self, _: &mut Ui) -> String { self.id.clone() }
	fn area(&self, ui: &mut Ui) -> Area { ui.window_area() }
	fn layer(&self, ui: &mut Ui) -> Layer { ui.paint_style().layer }
	fn begin(&mut self, _: &mut Ui, _: &mut Painter, _: &Response, _: &str) -> bool { true }
	fn end<R>(&mut self, ui: &mut Ui, painter: &mut Painter, inner_response: &InnerResponse<R>, id: &str) {
		let mut temp: MessageTemp = ui.memory_read(id).unwrap_or_default();
		painter.set_layer(Layer::Foreground);
		let mut messages = temp.messages.values_mut().collect::<Vec<&mut MessageInner>>();
		messages.par_sort_by(|a, b| a.create_time.elapsed().cmp(&b.create_time.elapsed()));
		let mut available_y = ui.style().space;
		let animation_time = Duration::milliseconds(250);
		let animation = Animation::new_standard(animation_time, Vec2::new(0.5, 0.0), Vec2::new(0.5, 1.0));
		for msg in messages {
			let background_color = if let Status::Default = msg.message.status {
				ui.style().primary_color
			}else {
				msg.message.status.into_color(ui)
			};
			if msg.message.text.color.is_none() {
				msg.message.text.color = Some(if background_color.difference(&Color::from(1.0)) > background_color.difference(&Color::from(0.0)) {
					1.0.into()
				}else {
					0.0.into()
				});
			}
			if msg.delete_time.is_none() && msg.message.should_delete {
				msg.delete_time = Some(Instant::now())
			}
			let duration = if msg.message.should_delete {
				msg.delete_time.unwrap().elapsed()
			} else {
				msg.create_time.elapsed()
			};
			let factor = animation.caculate(&duration).unwrap_or(1.0);
			let factor = (if msg.message.should_delete {
				1.0 - factor
			}else {
				factor
			} * 255.0) as u8;
			if let Some(t) = msg.message.sustain_time {
				if duration > t {
					msg.message.should_delete = true;
					msg.delete_time = Some(Instant::now());
				}
			}
			painter.set_color(background_color.set_alpha(factor));
			let text_area = msg.message.text_area(painter);
			let position = Vec2::new((ui.window_area().width() - text_area.width()) / 2.0 / painter.style().scale_factor, available_y) + inner_response.response.area.left_top();
			painter.set_position(position - Vec2::new(ui.style().space, 0.0));
			painter.rect(text_area.width_and_height() + Vec2::new(ui.style().space * 2.0, ui.style.space), Vec2::same(5.0));
			let message_color = msg.message.get_color(ui).set_alpha(factor);
			msg.message = msg.message.clone().set_color(message_color);
			msg.message.text_draw(painter, position + Vec2::new(0.0, ui.style().space / 2.0), ui);
			available_y = available_y + text_area.height() + ui.style().space * 2.0;
		};
		temp.messages.retain(|_, msg| {
			!(msg.message.should_delete && msg.delete_time.unwrap().elapsed() > Duration::milliseconds(250))
		});
		ui.memory_save(id, &temp);
	}
	fn is_clickable(&self, _: &mut Ui) -> bool { false }
	fn is_dragable(&self, _: &mut Ui) -> bool { false }
}