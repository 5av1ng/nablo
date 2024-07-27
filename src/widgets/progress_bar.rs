use time::Duration;
use crate::prelude::Animation;
use crate::widgets::Color;
use nablo_shape::prelude::Vec2;
use crate::Painter;
use nablo_shape::prelude::Area;
use crate::Ui;
use crate::Response;
use crate::Widget;
use crate::Instant;
use crate::prelude::Num;
use crate::prelude::Status;
use crate::prelude::ProgressBar;

#[derive(serde::Deserialize, serde::Serialize, Default, Debug)]
struct ProgressBarTemp {
	last_status: Status,
	last_color: Color,
	status_change_time: Vec<Instant>,
	to_progress_value: f64,
	from_progress_value: f64,
	progress_change_time: Vec<Instant>,
}

impl ProgressBar {
	/// get a new [`ProgressBar`], 1.0 for finished
	pub fn new(progress: impl Num) -> Self {
		Self {
			progress: progress.to_f64(),
			attach: false,
			status: Status::default(),
			width: 200.0
		}
	}

	/// change current progress
	pub fn progress(self, progress: impl Num) -> Self {
		Self {
			progress: progress.to_f64(),
			..self
		}
	}

	/// change current attachment, if attach is true, the progress bar will attached on the container
	pub fn attach(self, attach: bool) -> Self {
		Self {
			attach,
			..self
		}
	}

	/// change current status
	pub fn status(self, status: Status) -> Self {
		Self {
			status,
			..self
		}
	}

	/// change current width
	pub fn width(self, progress: impl Num) -> Self {
		Self {
			progress: progress.to_f64().clamp(0.0, 1.0),
			..self
		}
	}
}

impl Widget for ProgressBar {
	fn draw(&mut self, ui: &mut Ui, response: &Response, painter: &mut Painter) {
		// logic
		let mut temp: ProgressBarTemp = ui.memory_read(&response.id).unwrap_or(ProgressBarTemp {
			last_color: self.status.into_color(ui),
			last_status: self.status.clone(),
			from_progress_value: self.progress,
			to_progress_value: self.progress,
			..Default::default()
		});
		let animation_time = Duration::milliseconds(250);
		let animation = Animation::new_standard(animation_time, Vec2::new(0.3, 0.0), Vec2::new(0.7, 1.0));
		let animation_color = Animation::new_standard(animation_time, Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0));

		let color_display = if !temp.status_change_time.is_empty() {
			let delta = Instant::now() - temp.status_change_time[temp.status_change_time.len() - 1];
			animation_color.caculate(&delta).unwrap_or(1.0) * (temp.last_status.into_color(ui) - temp.last_color) + temp.last_color
		}else {
			temp.last_color = temp.last_status.into_color(ui);
			temp.last_status.into_color(ui)
		}.normalize();

		let progress_display = if !temp.progress_change_time.is_empty() {
			let delta = Instant::now() - temp.progress_change_time[temp.progress_change_time.len() - 1];
			animation.caculate(&delta).unwrap_or(1.0) as f64 * (temp.to_progress_value - temp.from_progress_value) + temp.from_progress_value
		}else {
			temp.from_progress_value = temp.to_progress_value;
			temp.to_progress_value
		}.clamp(0.0, 1.0);

		if temp.last_status != self.status {
			temp.last_color = color_display;
			temp.last_status = self.status.clone();
			temp.status_change_time.push(Instant::now());
		}
		if temp.to_progress_value != self.progress {
			temp.from_progress_value = progress_display;
			temp.to_progress_value = self.progress;
			temp.progress_change_time.push(Instant::now());
		}
		temp.status_change_time.retain(|inner| inner.elapsed() <= animation_time);
		temp.progress_change_time.retain(|inner| inner.elapsed() <= animation_time);

		ui.memory_save(&response.id, &temp);

		// draw
		if self.attach {
			let clip = painter.style().clip;
			painter.set_clip(Area::INF);
			painter.set_position(ui.window_crossed().left_top() + Vec2::x(ui.style().space));
			painter.set_color(color_display);
			painter.rect(Vec2::new((ui.window_crossed().width() - ui.style().space * 2.0) * progress_display as f32, 4.0), Vec2::same(2.0));
			painter.set_clip(clip)
		}else {
			painter.set_position(Vec2::new(response.area.left_top().x, (response.area.center().y) - 8.0));
			painter.set_color(ui.style().slider_reached_color);
			painter.rect(Vec2::new(response.area.width(), 8.0), Vec2::same(4.0));
			painter.set_color(color_display);
			painter.rect(Vec2::new(response.area.width() * progress_display as f32, 8.0), Vec2::same(4.0));
		}
	}

	fn ui(&mut self, ui: &mut Ui, area: Option<Area>) -> Response {
		let area = if self.attach {
			Area::ZERO
		}else {
			area.unwrap_or([ui.available_position(), ui.available_position() + Vec2::new(self.width, ui.style.space)].into())
		};
		ui.response(area, false, false)
	}
}