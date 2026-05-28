use std::collections::HashSet;

use oas3::spec::{BooleanSchema, ObjectOrReference, ObjectSchema, SchemaType, SchemaTypeSet};

use crate::error::Error;
use crate::ir::{
    DiscriminatorDef, EnumDef, Field, PrimitiveType, StringEnumDef, StructDef, TypeDef,
    TypeDefKind, TypeRef, Variant,
};

/// Carries state needed when lowering schemas that may contain inline objects.
///
/// `existing_names` is the immutable set of top-level component-schema names.
/// `synthesized` collects new `TypeDef`s produced by promoting inline objects.
/// `name_stack` is the path of containing names; e.g. `["Parent", "Child"]`
/// while lowering `Parent.properties.child`. The PascalCase concatenation of
/// the stack is used as the candidate name for promotion.
///
/// `promotion_allowed`: schema-side lowering may promote inline objects to
/// named structs; operations-side must keep erroring. Consulted in Task 3
/// where synthesize_object is added.
struct LoweringCtx<'a> {
    existing_names: &'a HashSet<String>,
    synthesized: Vec<TypeDef>,
    name_stack: Vec<String>,
    promotion_allowed: bool,
}

impl<'a> LoweringCtx<'a> {
    fn new(existing_names: &'a HashSet<String>) -> Self {
        Self {
            existing_names,
            synthesized: Vec::new(),
            name_stack: Vec::new(),
            promotion_allowed: true,
        }
    }

    /// Build a ctx for the operations-side path, where promotion is forbidden.
    /// Uses a static empty set because the operations path never needs name-collision
    /// checks (it errors on inline objects regardless).
    fn forbidden() -> Self {
        static EMPTY: std::sync::OnceLock<HashSet<String>> = std::sync::OnceLock::new();
        let existing = EMPTY.get_or_init(HashSet::new);
        Self {
            existing_names: existing,
            synthesized: Vec::new(),
            name_stack: Vec::new(),
            promotion_allowed: false,
        }
    }

    fn push(&mut self, segment: &str) {
        self.name_stack.push(pascal_case(segment));
    }

    fn pop(&mut self) {
        self.name_stack.pop();
    }
}

/// Convert `snake_case`, `kebab-case`, or `camelCase` to `PascalCase`.
/// Already-PascalCase input is left unchanged.
fn pascal_case(s: &str) -> String {
    let mut out = String::new();
    let mut capitalize_next = true;
    for c in s.chars() {
        if c == '_' || c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            out.extend(c.to_uppercase());
            capitalize_next = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// Build a `TypeDef` for an inline `type: object` schema and register it in `ctx`.
/// Returns a `TypeRef::Named` pointing at the freshly synthesised type.
///
/// `array_item` distinguishes array-items from struct properties: when true the
/// current top-of-stack name is suffixed with `Item` (so `Catalogue.entries`
/// items become `CatalogueEntriesItem`, not `CatalogueEntries`).
///
/// When `ctx.promotion_allowed` is false (operations-side calls), this raises
/// the same `UnsupportedSchema` error that pre-Task-3 code did. Promotion is
/// scoped to component schemas only.
fn synthesize_object(
    ctx: &mut LoweringCtx,
    schema: &ObjectSchema,
    array_item: bool,
) -> Result<TypeRef, Error> {
    if !ctx.promotion_allowed {
        return Err(Error::UnsupportedSchema {
            context: "inline object with additionalProperties: false — use a $ref to a named schema instead".to_string(),
        });
    }

    let mut base_name = ctx.name_stack.join("");
    if array_item {
        base_name.push_str("Item");
    }
    let name = unique_name(ctx, &base_name);

    let description = schema.description.clone();

    let mut bases = Vec::new();
    let mut fields = Vec::new();
    for entry in &schema.all_of {
        match entry {
            ObjectOrReference::Ref { ref_path, .. } => {
                bases.push(TypeRef::Named(extract_schema_name(ref_path)?));
            }
            ObjectOrReference::Object(inline) => {
                lower_properties_into(ctx, &mut fields, inline)?;
            }
        }
    }
    lower_properties_into(ctx, &mut fields, schema)?;

    let deny_unknown_fields = matches!(
        &schema.additional_properties,
        Some(oas3::spec::Schema::Boolean(BooleanSchema(false)))
    );

    ctx.synthesized.push(TypeDef {
        name: name.clone(),
        description,
        kind: TypeDefKind::Struct(StructDef {
            bases,
            fields,
            deny_unknown_fields,
        }),
    });

    Ok(TypeRef::Named(name))
}

/// Pick a name that doesn't clash with an existing component schema or a
/// previously synthesised type. Tries `base`, then `base2`, `base3`, ...
fn unique_name(ctx: &LoweringCtx, base: &str) -> String {
    let taken = |candidate: &str| -> bool {
        ctx.existing_names.contains(candidate)
            || ctx.synthesized.iter().any(|t| t.name == candidate)
    };
    if !taken(base) {
        return base.to_string();
    }
    let mut n = 2;
    loop {
        let candidate = format!("{base}{n}");
        if !taken(&candidate) {
            return candidate;
        }
        n += 1;
    }
}

pub fn lower_schemas(spec: &oas3::Spec) -> Result<Vec<TypeDef>, Error> {
    let Some(components) = &spec.components else {
        return Ok(Vec::new());
    };

    let existing_names: HashSet<String> = components.schemas.keys().cloned().collect();
    let mut ctx = LoweringCtx::new(&existing_names);

    let mut types: Vec<TypeDef> = components
        .schemas
        .iter()
        .map(|(name, schema_or_ref)| lower_top_level(&mut ctx, name, schema_or_ref))
        .collect::<Result<_, _>>()?;

    types.extend(ctx.synthesized);

    super::box_cycles::box_recursive_types(&mut types);

    Ok(types)
}

fn lower_top_level(
    ctx: &mut LoweringCtx,
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
        ObjectOrReference::Object(schema) => {
            ctx.push(name);
            let result = lower_schema(ctx, name, schema);
            ctx.pop();
            result
        }
    }
}

fn lower_schema(
    ctx: &mut LoweringCtx,
    name: &str,
    schema: &ObjectSchema,
) -> Result<TypeDef, Error> {
    let description = schema.description.clone();

    // string enum: type string + enum values
    if (is_string_type(schema) || schema.schema_type.is_none()) && !schema.enum_values.is_empty() {
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
        return lower_composite(ctx, name, schema, has_structure, has_variants);
    }

    // array: type array + items (or prefixItems for tuples)
    if is_array_type(schema) {
        let type_ref = lower_array_type(ctx, schema)?;
        return Ok(TypeDef {
            name: name.to_string(),
            description,
            kind: TypeDefKind::Alias(type_ref),
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

    // top-level schema with only additionalProperties
    if let Some(additional) = &schema.additional_properties {
        return match lower_additional_properties(additional)? {
            Some(type_ref) => Ok(TypeDef {
                name: name.to_string(),
                description,
                kind: TypeDefKind::Alias(type_ref),
            }),
            None => Ok(TypeDef {
                name: name.to_string(),
                description,
                kind: TypeDefKind::Struct(StructDef {
                    bases: vec![],
                    fields: vec![],
                    deny_unknown_fields: true,
                }),
            }),
        };
    }

    // empty schema: no type, no structure, no constraints — represents any JSON value
    Ok(TypeDef {
        name: name.to_string(),
        description,
        kind: TypeDefKind::Alias(TypeRef::Any),
    })
}

fn lower_composite(
    ctx: &mut LoweringCtx,
    name: &str,
    schema: &ObjectSchema,
    has_structure: bool,
    has_variants: bool,
) -> Result<TypeDef, Error> {
    let description = schema.description.clone();

    // pure enum: only oneOf/anyOf, no fields or bases
    if has_variants && !has_structure {
        let (variants, discriminator) = lower_variants(ctx, schema)?;
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
                lower_properties_into(ctx, &mut fields, inline)?;
            }
        }
    }

    lower_properties_into(ctx, &mut fields, schema)?;

    let deny_unknown_fields = matches!(
        &schema.additional_properties,
        Some(oas3::spec::Schema::Boolean(BooleanSchema(false)))
    );

    Ok(TypeDef {
        name: name.to_string(),
        description,
        kind: TypeDefKind::Struct(StructDef {
            bases,
            fields,
            deny_unknown_fields,
        }),
    })
}

fn lower_variants(
    ctx: &mut LoweringCtx,
    schema: &ObjectSchema,
) -> Result<(Vec<Variant>, Option<DiscriminatorDef>), Error> {
    let discriminator = schema.discriminator.as_ref().map(|d| DiscriminatorDef {
        property: d.property_name.clone(),
        mapping: d.mapping.clone().unwrap_or_default(),
    });

    // Collect TypeRefs for each oneOf/anyOf entry in source order. We use them
    // both for the no-mapping path and for filling in unmapped branches below.
    let mut branch_refs: Vec<TypeRef> = Vec::new();
    for entry in schema.one_of.iter().chain(schema.any_of.iter()) {
        branch_refs.push(lower_type_ref_in_schema(ctx, entry)?);
    }

    let Some(disc) = &discriminator else {
        // No discriminator: one variant per branch, no wire tags.
        let variants = branch_refs
            .into_iter()
            .map(|type_ref| Variant {
                type_ref,
                mapping_value: None,
            })
            .collect();
        return Ok((variants, discriminator));
    };

    if disc.mapping.is_empty() {
        // Discriminator without an explicit mapping: one variant per branch;
        // the wire tag is the schema name (handled by codegen via type_ref).
        let variants = branch_refs
            .into_iter()
            .map(|type_ref| Variant {
                type_ref,
                mapping_value: None,
            })
            .collect();
        return Ok((variants, discriminator));
    }

    // Mapping present. Build one variant per mapping key, in BTreeMap (alpha)
    // order. Keys whose target is not in the oneOf list are silently dropped
    // (matches prior behaviour and OpenAPI's permissive stance on stray keys).
    let mut variants: Vec<Variant> = Vec::new();
    let mut covered_refs: Vec<TypeRef> = Vec::new();
    for (key, schema_ref) in &disc.mapping {
        let target = match extract_schema_name(schema_ref) {
            Ok(name) => TypeRef::Named(name),
            Err(_) => continue,
        };
        if !branch_refs.contains(&target) {
            continue;
        }
        if !covered_refs.contains(&target) {
            covered_refs.push(target.clone());
        }
        variants.push(Variant {
            type_ref: target,
            mapping_value: Some(key.clone()),
        });
    }

    // Append any oneOf branches that no mapping key referred to. They keep
    // the implicit-schema-name behaviour. We deliberately allow a branch to
    // be both mapped and listed without a key only if mapping coverage is
    // partial; once a TypeRef is covered by at least one mapping key, we do
    // not also emit an "implicit" variant for it.
    for type_ref in branch_refs {
        if covered_refs.contains(&type_ref) {
            continue;
        }
        variants.push(Variant {
            type_ref,
            mapping_value: None,
        });
    }

    Ok((variants, discriminator))
}

fn lower_properties_into(
    ctx: &mut LoweringCtx,
    fields: &mut Vec<Field>,
    schema: &ObjectSchema,
) -> Result<(), Error> {
    for (prop_name, prop_schema) in &schema.properties {
        ctx.push(prop_name);
        let type_ref = lower_type_ref_in_schema(ctx, prop_schema);
        ctx.pop();
        let type_ref = type_ref?;
        fields.push(Field {
            name: prop_name.clone(),
            type_ref,
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
    let mut ctx = LoweringCtx::forbidden();
    lower_type_ref_in_schema(&mut ctx, schema_or_ref)
}

fn lower_type_ref_in_schema(
    ctx: &mut LoweringCtx,
    schema_or_ref: &ObjectOrReference<ObjectSchema>,
) -> Result<TypeRef, Error> {
    match schema_or_ref {
        ObjectOrReference::Ref { ref_path, .. } => {
            Ok(TypeRef::Named(extract_schema_name(ref_path)?))
        }
        ObjectOrReference::Object(schema) => lower_inline_type(ctx, schema),
    }
}

fn lower_inline_type(ctx: &mut LoweringCtx, schema: &ObjectSchema) -> Result<TypeRef, Error> {
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
            let inner = lower_inline_type(ctx, &inner_schema)?;
            return Ok(TypeRef::Option(Box::new(inner)));
        }
    }

    // nullable: oneOf with exactly [{type: "null"}, {$ref or inline type}]
    if let Some(inner) = try_lower_oneof_nullable(ctx, schema)? {
        return Ok(TypeRef::Option(Box::new(inner)));
    }

    let Some(SchemaTypeSet::Single(schema_type)) = &schema.schema_type else {
        // object with additionalProperties but no explicit type
        if let Some(additional) = &schema.additional_properties {
            if let Some(type_ref) = lower_additional_properties(additional)? {
                return Ok(type_ref);
            }
            // additionalProperties: false on a typeless inline schema — promote
            // it to a named struct.
            return synthesize_object(ctx, schema, /* array_item */ false);
        }
        // empty inline schema: no type, no additionalProperties — any JSON value
        if schema.properties.is_empty()
            && schema.all_of.is_empty()
            && schema.one_of.is_empty()
            && schema.any_of.is_empty()
            && schema.enum_values.is_empty()
        {
            return Ok(TypeRef::Any);
        }
        return Err(Error::UnsupportedSchema {
            context: format!("inline schema with no type: {schema:?}"),
        });
    };

    match schema_type {
        SchemaType::String => match schema.format.as_deref() {
            Some("date") => Ok(TypeRef::Primitive(PrimitiveType::Date)),
            Some("date-time") => Ok(TypeRef::Primitive(PrimitiveType::DateTime)),
            Some("uuid") => Ok(TypeRef::Primitive(PrimitiveType::Uuid)),
            _ => Ok(TypeRef::Primitive(PrimitiveType::String)),
        },
        SchemaType::Boolean => Ok(TypeRef::Primitive(PrimitiveType::Bool)),
        SchemaType::Integer => match schema.format.as_deref() {
            Some("int64") => Ok(TypeRef::Primitive(PrimitiveType::I64)),
            _ => Ok(TypeRef::Primitive(PrimitiveType::I32)),
        },
        SchemaType::Number => match schema.format.as_deref() {
            Some("float") => Ok(TypeRef::Primitive(PrimitiveType::F32)),
            _ => Ok(TypeRef::Primitive(PrimitiveType::F64)),
        },
        SchemaType::Array => lower_array_type(ctx, schema),
        SchemaType::Object => {
            if let Some(additional) = &schema.additional_properties {
                if let Some(type_ref) = lower_additional_properties(additional)? {
                    return Ok(type_ref);
                }
                // additionalProperties: false with properties or empty — synthesise.
                synthesize_object(ctx, schema, /* array_item */ false)
            } else {
                // type: object with `properties` but no `additionalProperties`.
                synthesize_object(ctx, schema, /* array_item */ false)
            }
        }
        SchemaType::Null => Ok(TypeRef::Option(Box::new(TypeRef::Primitive(
            PrimitiveType::String,
        )))),
    }
}

/// Lower an array schema to either an `Array` (open-ended) or `Tuple` (bounded `prefixItems`).
fn lower_array_type(ctx: &mut LoweringCtx, schema: &ObjectSchema) -> Result<TypeRef, Error> {
    if !schema.prefix_items.is_empty() {
        return lower_prefix_items(ctx, schema);
    }
    let inner = lower_items(ctx, schema)?;
    Ok(TypeRef::Array(Box::new(inner)))
}

/// Lower a `prefixItems` array to `TypeRef::Tuple`. Requires the array length be pinned
/// by `minItems == maxItems == prefixItems.len()` — otherwise the schema permits arrays
/// shorter or longer than the prefix and a fixed-size tuple would misrepresent the wire
/// shape.
fn lower_prefix_items(ctx: &mut LoweringCtx, schema: &ObjectSchema) -> Result<TypeRef, Error> {
    let n = schema.prefix_items.len() as u64;
    if schema.min_items != Some(n) || schema.max_items != Some(n) {
        return Err(Error::UnsupportedSchema {
            context: format!(
                "prefixItems with {n} entries requires minItems and maxItems both equal to {n}"
            ),
        });
    }
    let mut items = Vec::new();
    for item in &schema.prefix_items {
        items.push(lower_type_ref_in_schema(ctx, item)?);
    }
    Ok(TypeRef::Tuple(items))
}

fn lower_items(ctx: &mut LoweringCtx, schema: &ObjectSchema) -> Result<TypeRef, Error> {
    let Some(items) = &schema.items else {
        return Err(Error::UnsupportedSchema {
            context: "array without items".to_string(),
        });
    };
    match items.as_ref() {
        oas3::spec::Schema::Object(inner) => match &**inner {
            ObjectOrReference::Ref { ref_path, .. } => {
                Ok(TypeRef::Named(extract_schema_name(ref_path)?))
            }
            ObjectOrReference::Object(obj) => {
                if is_inline_object(obj) {
                    synthesize_object(ctx, obj, /* array_item */ true)
                } else {
                    lower_inline_type(ctx, obj)
                }
            }
        },
        oas3::spec::Schema::Boolean(_) => Err(Error::UnsupportedSchema {
            context: "boolean schema in items".to_string(),
        }),
    }
}

/// True for schemas that `synthesize_object` would promote — used by
/// `lower_items` to decide whether to apply the `Item` suffix.
fn is_inline_object(schema: &ObjectSchema) -> bool {
    let is_explicit_object = matches!(
        schema.schema_type,
        Some(SchemaTypeSet::Single(SchemaType::Object))
    );
    let is_implicit_closed = schema.schema_type.is_none()
        && matches!(
            &schema.additional_properties,
            Some(oas3::spec::Schema::Boolean(BooleanSchema(false)))
        );
    is_explicit_object || is_implicit_closed
}

fn lower_additional_properties(schema: &oas3::spec::Schema) -> Result<Option<TypeRef>, Error> {
    match schema {
        oas3::spec::Schema::Object(inner) => {
            let value_type = lower_type_ref(inner)?;
            Ok(Some(TypeRef::Map(Box::new(value_type))))
        }
        oas3::spec::Schema::Boolean(BooleanSchema(true)) => Ok(Some(TypeRef::Map(Box::new(
            TypeRef::Primitive(PrimitiveType::String),
        )))),
        oas3::spec::Schema::Boolean(BooleanSchema(false)) => Ok(None),
    }
}

fn try_lower_primitive(schema: &ObjectSchema) -> Option<TypeRef> {
    let SchemaTypeSet::Single(t) = schema.schema_type.as_ref()? else {
        return None;
    };
    match t {
        SchemaType::String => match schema.format.as_deref() {
            Some("date") => Some(TypeRef::Primitive(PrimitiveType::Date)),
            Some("date-time") => Some(TypeRef::Primitive(PrimitiveType::DateTime)),
            Some("uuid") => Some(TypeRef::Primitive(PrimitiveType::Uuid)),
            _ => Some(TypeRef::Primitive(PrimitiveType::String)),
        },
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

fn try_lower_oneof_nullable(
    ctx: &mut LoweringCtx,
    schema: &ObjectSchema,
) -> Result<Option<TypeRef>, Error> {
    if schema.one_of.len() != 2 {
        return Ok(None);
    }

    let is_null_variant = |entry: &ObjectOrReference<ObjectSchema>| -> bool {
        matches!(
            entry,
            ObjectOrReference::Object(s)
                if matches!(s.schema_type, Some(SchemaTypeSet::Single(SchemaType::Null)))
                    && s.properties.is_empty()
                    && s.one_of.is_empty()
                    && s.any_of.is_empty()
                    && s.all_of.is_empty()
                    && s.enum_values.is_empty()
        )
    };

    let (null_idx, other_idx) = if is_null_variant(&schema.one_of[0]) {
        (0, 1)
    } else if is_null_variant(&schema.one_of[1]) {
        (1, 0)
    } else {
        return Ok(None);
    };
    let _ = null_idx;

    let inner = lower_type_ref_in_schema(ctx, &schema.one_of[other_idx])?;
    Ok(Some(inner))
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
