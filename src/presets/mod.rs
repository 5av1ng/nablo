/*! `nablo` build-in templates
 * */

use crate::prelude::*;
use crate::Ui;
use anyhow::*;

/// display a tree of your struct
pub fn settings<'a, T: serde::Serialize + serde::Deserialize<'a>>(input: &mut T, id: String, ui: &mut Ui) -> Result<()> {
	let mut data = to_data(input)?;
	setting_inner(&mut data, id, ui);
	*input = from_data(&mut data)?;

	Ok(())
}

fn setting_inner(input: &mut ParsedData, id: String, ui: &mut Ui) {
	let name = input.name.clone();
	match &mut input.data {
		DataEnum::Node(node) => {
			ui.show(&mut Collapsing::new(id.clone() + &name).set_text(name.clone()), |ui, _| {
				for inner in node {
					setting_inner(inner, id.clone() + &name, ui);
				}
			});
		},
		DataEnum::Map(map) => {
			let (mut inner1, mut inner2) = *map.clone();
			setting_inner(&mut inner1, id.clone() + &name + "1", ui);
			setting_inner(&mut inner2, id + &name + "2", ui);
			*map = Box::new((inner1, inner2));
		},
		DataEnum::Enum(enum_string, node) => {
			ui.label(enum_string.clone());
			ui.show(&mut Collapsing::new(id.clone() + &name).set_text(name.clone()), |ui, _| {
				for inner in node {
					setting_inner(inner, id.clone() + &name, ui);
				}
			});
		},
		DataEnum::Data(_) => {},
		DataEnum::String(inner) => {
			ui.horizental(|ui| {
				ui.label(name.clone());
				ui.label(inner.clone());
			});
		},
		DataEnum::Int(num, non_neg) => {
			ui.add(DragableValue::new(num).non_negative(!*non_neg).set_text(name.clone()));
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