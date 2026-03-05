use oas3::spec::{BooleanSchema, ObjectOrReference, ObjectSchema, SchemaType, SchemaTypeSet};

use crate::error::Error;
use crate::ir::{
    DiscriminatorDef, EnumDef, Field, PrimitiveType, StringEnumDef, StructDef, TypeDef,
    TypeDefKind, TypeRef, Variant,
};

pub fn lower_schemas(spec: &oas3::Spec) -> Result<Vec<TypeDef>, Error> {
    let Some(components) = &spec.components else {
        return Ok(Vec::new());
    };

    components
        .schemas
        .iter()
        .map(|(name, schema_or_ref)| lower_top_level(name, schema_or_ref))
        .collect()
}

fn lower_top_level(
    name: &str,
    schema_or_ref: &ObjectOrReference<ObjectSchema>,
) -> Result<TypeDef, Error> {
    match schema_or_ref {
        ObjectOrReference::Ref { ref_path, .. } => {
            let target = extract_schema_name(ref_path)?;
            Ok(TypeDef {
                name: name.to_string(),
                description: None,
                kind: TypeDefKind::Alias(TypeRef::Named(target)),
            })
        }
        ObjectOrReference::Object(schema) => lower_schema(name, schema),
    }
}

fn lower_schema(name: &str, schema: &ObjectSchema) -> Result<TypeDef, Error> {
    let description = schema.description.clone();

    // string enum: type string + enum values
    if is_string_type(schema) && !schema.enum_values.is_empty() {
        let values = schema
            .enum_values
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        return Ok(TypeDef {
            name: name.to_string(),
            description,
            kind: TypeDefKind::StringEnum(StringEnumDef { values }),
        });
    }

    // struct or enum: has properties, allOf, oneOf, or anyOf
    let has_structure = !schema.properties.is_empty() || !schema.all_of.is_empty();
    let has_variants = !schema.one_of.is_empty() || !schema.any_of.is_empty();

    if has_structure || has_variants {
        return lower_composite(name, schema, has_structure, has_variants);
    }

    // array: type array + items
    if is_array_type(schema) {
        let inner = lower_items(schema)?;
        return Ok(TypeDef {
            name: name.to_string(),
            description,
            kind: TypeDefKind::Alias(TypeRef::Array(Box::new(inner))),
        });
    }

    // primitive type alias
    if let Some(type_ref) = try_lower_primitive(schema) {
        return Ok(TypeDef {
            name: name.to_string(),
            description,
            kind: TypeDefKind::Alias(type_ref),
        });
    }

    Err(Error::UnsupportedSchema {
        context: format!("schema '{name}'"),
    })
}

fn lower_composite(
    name: &str,
    schema: &ObjectSchema,
    has_structure: bool,
    has_variants: bool,
) -> Result<TypeDef, Error> {
    let description = schema.description.clone();

    // pure enum: only oneOf/anyOf, no fields or bases
    if has_variants && !has_structure {
        let (variants, discriminator) = lower_variants(schema)?;
        return Ok(TypeDef {
            name: name.to_string(),
            description,
            kind: TypeDefKind::Enum(EnumDef {
                variants,
                discriminator,
            }),
        });
    }

    // struct (possibly with bases from allOf)
    let mut bases = Vec::new();
    let mut fields = Vec::new();

    for entry in &schema.all_of {
        match entry {
            ObjectOrReference::Ref { ref_path, .. } => {
                bases.push(TypeRef::Named(extract_schema_name(ref_path)?));
            }
            ObjectOrReference::Object(inline) => {
                lower_properties_into(&mut fields, inline)?;
            }
        }
    }

    lower_properties_into(&mut fields, schema)?;

    Ok(TypeDef {
        name: name.to_string(),
        description,
        kind: TypeDefKind::Struct(StructDef { bases, fields }),
    })
}

fn lower_variants(
    schema: &ObjectSchema,
) -> Result<(Vec<Variant>, Option<DiscriminatorDef>), Error> {
    let mut variants = Vec::new();
    for entry in schema.one_of.iter().chain(schema.any_of.iter()) {
        variants.push(Variant {
            type_ref: lower_type_ref(entry)?,
            mapping_value: None,
        });
    }

    let discriminator = schema.discriminator.as_ref().map(|d| DiscriminatorDef {
        property: d.property_name.clone(),
        mapping: d.mapping.clone().unwrap_or_default(),
    });

    if let Some(disc) = &discriminator {
        for (value, schema_ref) in &disc.mapping {
            let target = extract_schema_name(schema_ref).unwrap_or_else(|_| schema_ref.clone());
            if let Some(variant) = variants
                .iter_mut()
                .find(|v| v.type_ref == TypeRef::Named(target.clone()))
            {
                variant.mapping_value = Some(value.clone());
            }
        }
    }

    Ok((variants, discriminator))
}

fn lower_properties_into(fields: &mut Vec<Field>, schema: &ObjectSchema) -> Result<(), Error> {
    for (prop_name, prop_schema) in &schema.properties {
        fields.push(Field {
            name: prop_name.clone(),
            type_ref: lower_type_ref(prop_schema)?,
            required: schema.required.contains(prop_name),
            description: match prop_schema {
                ObjectOrReference::Object(s) => s.description.clone(),
                _ => None,
            },
        });
    }
    Ok(())
}

pub(crate) fn lower_type_ref(
    schema_or_ref: &ObjectOrReference<ObjectSchema>,
) -> Result<TypeRef, Error> {
    match schema_or_ref {
        ObjectOrReference::Ref { ref_path, .. } => {
            Ok(TypeRef::Named(extract_schema_name(ref_path)?))
        }
        ObjectOrReference::Object(schema) => lower_inline_type(schema),
    }
}

fn lower_inline_type(schema: &ObjectSchema) -> Result<TypeRef, Error> {
    // nullable: type is [T, "null"]
    if let Some(SchemaTypeSet::Multiple(types)) = &schema.schema_type {
        let non_null: Vec<_> = types
            .iter()
            .copied()
            .filter(|t| *t != SchemaType::Null)
            .collect();
        if non_null.len() == 1 && types.len() == 2 {
            let mut inner_schema = schema.clone();
            inner_schema.schema_type = Some(SchemaTypeSet::Single(non_null[0]));
            let inner = lower_inline_type(&inner_schema)?;
            return Ok(TypeRef::Option(Box::new(inner)));
        }
    }

    let Some(SchemaTypeSet::Single(schema_type)) = &schema.schema_type else {
        // object with additionalProperties but no explicit type
        if let Some(additional) = &schema.additional_properties {
            return lower_additional_properties(additional);
        }
        return Err(Error::UnsupportedSchema {
            context: format!("inline schema with no type: {schema:?}"),
        });
    };

    match schema_type {
        SchemaType::String => Ok(TypeRef::Primitive(PrimitiveType::String)),
        SchemaType::Boolean => Ok(TypeRef::Primitive(PrimitiveType::Bool)),
        SchemaType::Integer => match schema.format.as_deref() {
            Some("int64") => Ok(TypeRef::Primitive(PrimitiveType::I64)),
            _ => Ok(TypeRef::Primitive(PrimitiveType::I32)),
        },
        SchemaType::Number => match schema.format.as_deref() {
            Some("float") => Ok(TypeRef::Primitive(PrimitiveType::F32)),
            _ => Ok(TypeRef::Primitive(PrimitiveType::F64)),
        },
        SchemaType::Array => {
            let inner = lower_items(schema)?;
            Ok(TypeRef::Array(Box::new(inner)))
        }
        SchemaType::Object => {
            if let Some(additional) = &schema.additional_properties {
                lower_additional_properties(additional)
            } else {
                Err(Error::UnsupportedSchema {
                    context: "inline object without additionalProperties".to_string(),
                })
            }
        }
        SchemaType::Null => Ok(TypeRef::Option(Box::new(TypeRef::Primitive(
            PrimitiveType::String,
        )))),
    }
}

fn lower_items(schema: &ObjectSchema) -> Result<TypeRef, Error> {
    let Some(items) = &schema.items else {
        return Err(Error::UnsupportedSchema {
            context: "array without items".to_string(),
        });
    };
    match items.as_ref() {
        oas3::spec::Schema::Object(inner) => lower_type_ref(inner),
        oas3::spec::Schema::Boolean(_) => Err(Error::UnsupportedSchema {
            context: "boolean schema in items".to_string(),
        }),
    }
}

fn lower_additional_properties(schema: &oas3::spec::Schema) -> Result<TypeRef, Error> {
    match schema {
        oas3::spec::Schema::Object(inner) => {
            let value_type = lower_type_ref(inner)?;
            Ok(TypeRef::Map(Box::new(value_type)))
        }
        oas3::spec::Schema::Boolean(BooleanSchema(true)) => Ok(TypeRef::Map(Box::new(
            TypeRef::Primitive(PrimitiveType::String),
        ))),
        oas3::spec::Schema::Boolean(BooleanSchema(false)) => Err(Error::UnsupportedSchema {
            context: "additionalProperties: false".to_string(),
        }),
    }
}

fn try_lower_primitive(schema: &ObjectSchema) -> Option<TypeRef> {
    let SchemaTypeSet::Single(t) = schema.schema_type.as_ref()? else {
        return None;
    };
    match t {
        SchemaType::String => Some(TypeRef::Primitive(PrimitiveType::String)),
        SchemaType::Boolean => Some(TypeRef::Primitive(PrimitiveType::Bool)),
        SchemaType::Integer => match schema.format.as_deref() {
            Some("int64") => Some(TypeRef::Primitive(PrimitiveType::I64)),
            _ => Some(TypeRef::Primitive(PrimitiveType::I32)),
        },
        SchemaType::Number => match schema.format.as_deref() {
            Some("float") => Some(TypeRef::Primitive(PrimitiveType::F32)),
            _ => Some(TypeRef::Primitive(PrimitiveType::F64)),
        },
        _ => None,
    }
}

pub(crate) fn extract_schema_name(ref_path: &str) -> Result<String, Error> {
    ref_path
        .strip_prefix("#/components/schemas/")
        .map(String::from)
        .ok_or_else(|| Error::UnresolvedRef {
            ref_path: ref_path.to_string(),
        })
}

fn is_string_type(schema: &ObjectSchema) -> bool {
    matches!(
        schema.schema_type,
        Some(SchemaTypeSet::Single(SchemaType::String))
    )
}

fn is_array_type(schema: &ObjectSchema) -> bool {
    matches!(
        schema.schema_type,
        Some(SchemaTypeSet::Single(SchemaType::Array))
    )
}
