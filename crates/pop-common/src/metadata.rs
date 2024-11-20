// SPDX-License-Identifier: GPL-3.0

use scale_info::{form::PortableForm, PortableRegistry, Type, TypeDef, TypeDefPrimitive};

/// Formats a specified type, using the registry to output its full type representation.
///
/// # Arguments
/// * `ty`: A reference to the `Type<PortableForm>` to be formatted.
/// * `registry`: A reference to the `PortableRegistry` to resolve type dependencies.
pub fn format_type(ty: &Type<PortableForm>, registry: &PortableRegistry) -> String {
	let mut name = ty
		.path
		.segments
		.last()
		.map(|s| s.to_owned())
		.unwrap_or_else(|| ty.path.to_string());

	if !ty.type_params.is_empty() {
		let params: Vec<_> = ty
			.type_params
			.iter()
			.filter_map(|p| registry.resolve(p.ty.unwrap().id))
			.map(|t| format_type(t, registry))
			.collect();
		name = format!("{name}<{}>", params.join(","));
	}

	name = format!(
		"{name}{}",
		match &ty.type_def {
			TypeDef::Composite(composite) => {
				if composite.fields.is_empty() {
					return "".to_string();
				}

				let mut named = false;
				let fields: Vec<_> = composite
					.fields
					.iter()
					.filter_map(|f| match f.name.as_ref() {
						None => registry.resolve(f.ty.id).map(|t| format_type(t, registry)),
						Some(field) => {
							named = true;
							f.type_name.as_ref().map(|t| format!("{field}: {t}"))
						},
					})
					.collect();
				match named {
					true => format!(" {{ {} }}", fields.join(", ")),
					false => format!(" ({})", fields.join(", ")),
				}
			},
			TypeDef::Variant(variant) => {
				let variants: Vec<_> = variant
					.variants
					.iter()
					.map(|v| {
						if v.fields.is_empty() {
							return v.name.clone();
						}

						let name = v.name.as_str();
						let mut named = false;
						let fields: Vec<_> = v
							.fields
							.iter()
							.filter_map(|f| match f.name.as_ref() {
								None => registry.resolve(f.ty.id).map(|t| format_type(t, registry)),
								Some(field) => {
									named = true;
									f.type_name.as_ref().map(|t| format!("{field}: {t}"))
								},
							})
							.collect();
						format!(
							"{name}{}",
							match named {
								true => format!("{{ {} }}", fields.join(", ")),
								false => format!("({})", fields.join(", ")),
							}
						)
					})
					.collect();
				format!(": {}", variants.join(", "))
			},
			TypeDef::Sequence(sequence) => {
				format!(
					"[{}]",
					format_type(
						registry.resolve(sequence.type_param.id).expect("sequence type not found"),
						registry
					)
				)
			},
			TypeDef::Array(array) => {
				format!(
					"[{};{}]",
					format_type(
						registry.resolve(array.type_param.id).expect("array type not found"),
						registry
					),
					array.len
				)
			},
			TypeDef::Tuple(tuple) => {
				let fields: Vec<_> = tuple
					.fields
					.iter()
					.filter_map(|p| registry.resolve(p.id))
					.map(|t| format_type(t, registry))
					.collect();
				format!("({})", fields.join(","))
			},
			TypeDef::Primitive(primitive) => {
				use TypeDefPrimitive::*;
				match primitive {
					Bool => "bool",
					Char => "char",
					Str => "str",
					U8 => "u8",
					U16 => "u16",
					U32 => "u32",
					U64 => "u64",
					U128 => "u128",
					U256 => "u256",
					I8 => "i8",
					I16 => "i16",
					I32 => "i32",
					I64 => "i64",
					I128 => "i128",
					I256 => "i256",
				}
				.to_string()
			},
			TypeDef::Compact(compact) => {
				format!(
					"Compact<{}>",
					format_type(
						registry.resolve(compact.type_param.id).expect("compact type not found"),
						registry
					)
				)
			},
			TypeDef::BitSequence(_) => {
				unimplemented!("bit sequence not currently supported")
			},
		}
	);

	name
}
