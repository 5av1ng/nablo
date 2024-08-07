/*! `nablo` build-in templates
 * */

use crate::prelude::*;
use crate::Ui;
use anyhow::*;

/// display a tree of your struct.
pub fn settings<'a, T: serde::Serialize + serde::Deserialize<'a>>(input: &mut T, id: impl Into<String>, ui: &mut Ui) -> Result<()> {
	let id = id.into();
	let mut data = to_data(input)?;
	setting_inner(&mut data, id, ui);
	*input = from_data(&mut data)?;

	Ok(())
}

/// display a tree of your struct, returns what was changed, only available for numeric values.
pub fn settings_with_delta<'a, T: serde::Serialize + serde::Deserialize<'a>>(input: &mut T, id: impl Into<String>, ui: &mut Ui) -> Result<HashMap<String, f64>> {
	let id = id.into();
	let mut data = to_data(input)?;
	let input_backup: T = from_data(&mut data.clone())?;
	setting_inner(&mut data, id, ui);
	*input = from_data(&mut data)?;

	Ok(caculate_delta(input, &input_backup)?)
}

fn setting_inner(input: &mut ParsedData, id: String, ui: &mut Ui) {
	let name = input.name.clone();
	let id = format!("{}%%{}", id, name);
	match &mut input.data {
		DataEnum::Node(node) => {
			if node.is_empty() {
				ui.label(&name);
			}else {
				ui.show(&mut Collapsing::new(id.clone()).set_text(name.clone()), |ui, _| {
					for inner in node {
						setting_inner(inner, id.clone(), ui);
					}
				});
			}
		},
		DataEnum::Map(map) => {
			let (mut inner1, mut inner2) = *map.clone();
			setting_inner(&mut inner1, id.clone() + "key", ui);
			setting_inner(&mut inner2, id + "value", ui);
			*map = Box::new((inner1, inner2));
		},
		DataEnum::Enum(enum_string, node) => {
			if node.is_empty() {
				ui.label(format!("{}: {}", name, enum_string));
			}else {
				ui.show(&mut Collapsing::new(id.clone()).set_text(format!("{}: {}", name, enum_string)), |ui, _| {
					for inner in node {
						setting_inner(inner, id.clone(), ui);
					}
				});
			}
		},
		DataEnum::Data(_) => {},
		DataEnum::String(inner) => {
			ui.horizental(|ui| {
				ui.label(name.clone());
				ui.single_input(inner);
			});
		},
		DataEnum::Int(num, range) => {
			ui.add(DragableValue::new(num).range(range.clone()).set_text(name.clone()));
		},
		DataEnum::Float(num) => {
			ui.add(DragableValue::new(num).set_text(name.clone()));
		},
		DataEnum::Bool(inner) => {
			ui.switch(inner, name);
		},
		DataEnum::None => {},
	}
}