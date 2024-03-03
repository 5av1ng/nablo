use time::Duration;
use nablo_shape::prelude::Animation;
use crate::prelude::ShapeElement;
use crate::Instant;
use crate::Vec2;
use crate::widgets::TextSetting;
use nablo_shape::prelude::shape_elements::Layer;
use crate::InnerResponse;
use crate::Response;
use crate::Ui;
use nablo_shape::prelude::Area;
use crate::Container;
use crate::Painter;
use crate::prelude::Text;
use crate::prelude::Collapsing;

impl Collapsing {
	/// create a new collasping area
	pub fn new(id: impl Into<Text>) -> Self {
		let text = id.into();
		Self {
			id: text.text.clone(),
			text,
			..Default::default()
		}
	}

	/// set icon for a collapsing
	pub fn icon(self,area: Vec2, painter: impl FnOnce(&mut Painter)) -> Self {
		let mut icon = Painter::default();
		icon.paint_area = Area::new(Vec2::ZERO, area);
		icon.set_clip(Area::new(Vec2::ZERO, area));
		painter(&mut icon);
		Self {
			icon: Some(icon),
			..self
		}
	}

	/// set default open for a collapsing, by default its false
	pub fn default_open(self, default_open: bool) -> Self {
		Self {
			default_open,
			..self
		}
	}

	/// set current open state for a collapsing.
	pub fn open(&self, is_open: bool, ui: &mut Ui) {
		let id = ui.container_id(self);
		let temp = CollapsingTemp {
			is_open,
			..ui.memory_read(&id).unwrap_or(CollapsingTemp::default())
		};
		ui.memory_save(&id, temp);
	}
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
struct CollapsingTemp {
	area: Vec2,
	is_open: bool,
	change_time: Vec<Instant>
}

impl Container for Collapsing {
	fn get_id(&self, _: &mut Ui) -> String {
		self.id.clone()
	}

	fn area(&self, ui: &mut Ui) -> Area {
		let id = ui.container_id(self);
		let mut painter = ui.painter();
		let text_area = self.text_area(&mut painter);
		let icon_area = if let Some(t) = &self.icon {
			t.paint_area
		}else {
			Area::new(Vec2::ZERO, Vec2::same(16.0))
		};
		let width_and_height = Vec2::new(text_area.width_and_height().x + icon_area.width_and_height().x, 
			*[text_area.width_and_height().y, icon_area.width_and_height().y, ui.style().space].iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
		let width_and_height = if let Some(temp) = ui.memory_read::<CollapsingTemp>(&id) {
			if temp.is_open {
				width_and_height + temp.area
			}else {
				width_and_height
			}
		}else {
			width_and_height
		};
		
		Area::new(ui.available_position(), ui.available_position() + width_and_height + Vec2::same(ui.style().space))
	}

	fn layer(&self, ui: &mut Ui) -> Layer {
		ui.painter().style().layer
	}

	fn begin(&mut self, ui: &mut Ui, painter: &mut Painter, response: &Response, id: &String) -> bool {
		// logic
		let mut temp: CollapsingTemp = ui.memory_read(id).unwrap_or(CollapsingTemp{
			is_open: self.default_open,
			..Default::default()
		});
		let text_area = self.text_area(painter);
		let icon_area = if let Some(t) = &self.icon {
			t.paint_area
		}else {
			Area::new(Vec2::ZERO, Vec2::same(16.0))
		};
		let width_and_height = Vec2::new(text_area.width_and_height().x + icon_area.width_and_height().x, 
			*[text_area.width_and_height().y, icon_area.width_and_height().y, ui.style().space].iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
		let area = Area::new(response.area.left_top(), response.area.left_top() + width_and_height);
		if ui.input().is_any_mouse_released() {
			if let Some(t) = ui.input().cursor_position() {
				if area.is_point_inside(&t) {
					temp.change_time.push(Instant::now());
					temp.is_open = !temp.is_open;
				}
			}
		}
		if temp.change_time.len() > 2 {
			temp.change_time.remove(0);
		}
		ui.memory_save(id, &temp);

		// animation caculate
		let animation_time = Duration::milliseconds(150);
		let animation = Animation::new_standard(animation_time, Vec2::new(0.3, 0.0), Vec2::new(0.7, 1.0));
		let rotate = if temp.change_time.len() == 2 {
			let delta = temp.change_time[0].elapsed() - temp.change_time[1].elapsed();
			let calc = if delta > animation_time {
				animation.caculate(&temp.change_time[1].elapsed()).unwrap_or_else(|| 1.0)
			}else {
				animation.caculate(&(delta + temp.change_time[1].elapsed())).unwrap_or_else(|| 1.0)
			};
			if temp.is_open {
				calc
			}else {
				1.0 - calc
			}
		}else if temp.change_time.len() == 1 {
			let calc = animation.caculate(&temp.change_time[0].elapsed()).unwrap_or_else(|| 1.0);
			if temp.is_open {
				calc
			}else {
				1.0 - calc
			}
		}else {
			0.0
		} * std::f32::consts::PI * 0.5;

		// paint
		if let None = self.icon {
			let mut icon = Painter::default();
			icon.paint_area = Area::new(Vec2::ZERO, Vec2::same(16.0));
			icon.set_clip(Area::new(Vec2::ZERO, Vec2::same(16.0)));
			icon.set_color(1.0);
			icon.draw(ShapeElement::Polygon(vec!(Vec2::new(4.0, 4.0), Vec2::new(4.0 * 3.0_f32.sqrt() + 4.0, 8.0), Vec2::new(4.0, 12.0)).into()));
			self.icon = Some(icon);
		}
		let mut icon = self.icon.clone().unwrap();
		let icon_area = icon.paint_area;
		let position = area.area[0] + Vec2::new(0.0, (area.height() - icon_area.height()) / 2.0);
		icon.change_transform_origin(icon_area.center());
		icon.move_delta_to(position);
		icon.change_clip(painter.style().clip);
		icon.change_rotate(rotate);
		icon.change_layer(painter.style().layer);
		painter.append(&mut icon);

		let text_area = self.text.text_area(painter);
		let position = area.area[0] + Vec2::new(icon_area.width(), (area.height() - text_area.height()) / 2.0);
		self.text.text_draw(painter, position, ui);

		painter.set_offset(Vec2::new(0.0, area.height()));

		temp.is_open
	}

	fn end<R>(&mut self, ui: &mut Ui, _: &mut Painter, inner_response: &InnerResponse<R>, id: &String) {
		let mut temp: CollapsingTemp = ui.memory_read(id).unwrap();
		if temp.is_open {
			let mut area = Area::ZERO;
			for res in &inner_response.inner_responses {
				if res.area != inner_response.response.area {
					area.combine(&res.area);
				}
			}
			temp.area = area.width_and_height();
		}else {
			temp.area = Vec2::ZERO
		}
		ui.memory_save(id, temp);
	}
	fn is_clickable(&self, _: &mut Ui) -> bool { true }
	fn is_dragable(&self, _: &mut Ui) -> bool { false }
}