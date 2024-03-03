use nablo_shape::math::Vec2;
use nablo::widgets::*;
use nablo::container::*;
use nablo::Manager;
use nablo::Ui;
use nablo::App;
#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Default)]
struct MyApp {
	position: Vec2,
	count: i32,
}

impl App for MyApp {
	fn app(&mut self, ui: &mut Ui) {
		if ui.add(Button::new("add")).is_clicked() {
			self.position = self.position + Vec2::new(20.0,20.0);
			self.count += 1;
		};
		if ui.add(Button::new("minus")).is_clicked() {
			self.position = self.position - Vec2::new(20.0,20.0);
			self.count -= 1;
		};
		let position_new = self.position.clone();
		ui.add(Canvas::new(Vec2::new(400.0, 200.0), |painter| {
			painter.set_color([100,100,100,100]);
			painter.rect(Vec2::new(400.0, 200.0), Vec2::same(0.0));
			painter.set_position(position_new);
			painter.set_color([255,255,255,255]);
			painter.cir(50.0);
		}));
		ui.add(Label::new(format!("{:?}", self.position)));
		ui.add(Label::new(format!("count {}", self.count)).underline(true));
	}
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
fn run() {
	Manager::new(MyApp::default()).run();
}

fn main() {
	run();
}
