use std::collections::HashSet;
use crate::prelude::*;

#[derive(serde::Deserialize, serde::Serialize, Default)]
struct TooltipProviderTemp {
	clicked_tooltip: HashSet<String>,
	area: Vec2,
}

impl TooltipProvider {
	/// create a new [`TooltipProvider`]
	pub fn new(id: impl Into<String>, tip: impl Into<Text>) -> Self {
		Self {
			id: id.into(),
			text: tip.into(),
			..Default::default()
		}
	}

	// /// change current [`TooltipProvider`] to click triggered
	// pub fn click_trigger(self, is_click_trigger: bool) -> Self {
	// 	Self {
	// 		is_click_trigger,
	// 		..self
	// 	}
	// }

	/// change current [`TooltipProvider`]'s align
	pub fn align(self, align: [Align; 2]) -> Self {
		Self {
			align,
			..self
		}
	}

	/// change current [`TooltipProvider`]'s tip
	pub fn tip(self, tip: impl Into<Text>) -> Self {
		Self {
			text: tip.into(),
			..self
		}
	}

	/// change how long do user hover that show's the tooltip
	pub fn hover_time(self, hover_time: Duration) -> Self {
		Self {
			hover_time,
			..self
		}
	}

	/// change current [`TooltipProvider`]'s padding
	pub fn padding(self, space: f32) -> Self {
		Self {
			space: Some(space),
			..self
		}
	}

	/// change current [`TooltipProvider`]'s status
	pub fn status(self, status: Status) -> Self {
		Self {
			status,
			..self
		}
	}
}

impl Container for TooltipProvider {
	fn get_id(&self, _: &mut Ui) -> String { self.id.clone() }
	fn is_clickable(&self, _: &mut Ui) -> bool { false }
	fn is_dragable(&self, _: &mut Ui) -> bool { false }
	fn area(&self, ui: &mut Ui) -> Area { 
		let id = ui.container_id(self);
		let temp:TooltipProviderTemp = ui.memory_read(&id).unwrap_or_default();
		Area::new(ui.available_position(), ui.available_position() + temp.area + Vec2::same(ui.style().space * 1.5)) 
	}
	fn layer(&self, ui: &mut Ui) -> Layer { ui.paint_style().layer }
	fn begin(&mut self, _: &mut Ui, _: &mut Painter, _: &Response, _: &str) -> bool { true }
	fn end<R>(&mut self, ui: &mut Ui, painter: &mut Painter, inner_response: &InnerResponse<R>, self_id: &str) {
		let mut temp:TooltipProviderTemp = ui.memory_read(self_id).unwrap_or_default(); 
		let animation_time = Duration::milliseconds(250);
		let animation = Animation::new_standard(animation_time, Vec2::new(0.5, 0.0), Vec2::new(0.5, 1.0));
		painter.set_layer(Layer::ToolTips);
		let mut area = Area::ZERO;
		for res in &inner_response.inner_responses {
			if res.id == self_id {
				continue;
			}
			if res.area != inner_response.response.area {
				area.combine(&res.area);
			}
			let tool_time_animated = if let Some(lost_hover_time) = res.lost_hovering_time() {
				if let Some(hover_time) = res.hovering_time(){
					let hover_time = hover_time - self.hover_time;
					if hover_time - lost_hover_time > animation_time {
						1.0 - animation.caculate(&lost_hover_time).unwrap_or(1.0)
					}else {
						animation.caculate(&(hover_time - lost_hover_time - lost_hover_time)).unwrap_or(0.0)
					}
				}else {
					0.0
				}
			}else if let Some(hover_time) = res.hovering_time() {
				let hover_time = hover_time - self.hover_time;
				animation.caculate(&hover_time).unwrap_or(1.0)
			}else {
				0.0
			};
			let background_color = if let Status::Default = self.status {
				ui.style().card_color.brighter(0.1)
			}else {
				self.status.into_color(ui)
			};
			if self.text.color.is_none() {
				self.text.color = Some(
					1.0.into()
				);
			}
			let alpha_factor = 0.7;
			let background_color = background_color.set_alpha(tool_time_animated * alpha_factor);
			let space = self.space.unwrap_or(ui.style().space);
			let text_area = self.text_area(painter);
			let height = text_area.height();
			let height = height + space;
			let width = text_area.width() + space * 2.0;
			let area = Area::new(ui.available_position(), ui.available_position() + Vec2::new(width, height));
			if tool_time_animated != 0.0 {
				let start_position = res.area.center() - Vec2::y(area.height() / 2.0);
				let final_position = {
					let x = match self.align[0] {
						Align::Left => res.area.left_top().x - ui.style().space / 2.0,
						Align::Middle => res.area.center().x,
						Align::Right => res.area.right_bottom().x + ui.style().space / 2.0,
					};
					let y = match self.align[1] {
						Align::Left => res.area.left_top().y - ui.style().space / 2.0 - area.height(),
						Align::Middle => res.area.center().y - area.height() / 2.0,
						Align::Right => res.area.right_bottom().y + ui.style().space / 2.0,
					};
					Vec2::new(x, y)
				};
				let position = start_position + (final_position - start_position) * tool_time_animated - Vec2::x(area.width() / 2.0);
				painter.set_position(position);
				painter.set_color(background_color);
				painter.rect(area.width_and_height(), Vec2::same(5.0));
				let text_width = if text_area.width() < area.width() {
					area.width()
				}else {
					text_area.width()
				};
				let position = position + Vec2::new(space, (area.height() - text_area.height()) / 2.0);
				if let Some(color) = &mut self.text.color {
					*color = color.set_alpha(tool_time_animated * alpha_factor)
				}
				self.text = self.text.clone().set_width(text_width);
				self.text.text_draw(painter, position, ui);
			}
		}
		temp.area = area.width_and_height();
		ui.memory_save(self_id, &temp);
	}
}