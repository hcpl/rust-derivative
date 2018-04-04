use syn;

/// Represent the `derivative` attributes on the input type (`struct`/`enum`).
#[derive(Debug, Default)]
pub struct Input {
    /// Whether `Clone` is present and its specific attributes.
    pub clone: Option<InputClone>,
    /// Whether `Copy` is present and its specific attributes.
    pub copy: Option<InputCopy>,
    /// Whether `Debug` is present and its specific attributes.
    pub debug: Option<InputDebug>,
    /// Whether `Default` is present and its specific attributes.
    pub default: Option<InputDefault>,
    /// Whether `Eq` is present and its specific attributes.
    pub eq: Option<InputEq>,
    /// Whether `Hash` is present and its specific attributes.
    pub hash: Option<InputHash>,
    /// Whether `Eq` is present and its specific attributes.
    pub partial_eq: Option<InputPartialEq>,
}

#[derive(Debug, Default)]
/// Represent the `derivative` attributes on a field.
pub struct Field {
    /// The parameters for `Clone`.
    clone: FieldClone,
    /// The parameters for `Copy`.
    copy_bound: Option<Vec<syn::WherePredicate>>,
    /// The parameters for `Debug`.
    debug: FieldDebug,
    /// The parameters for `Default`.
    default: FieldDefault,
    /// The parameters for `Eq`.
    eq_bound: Option<Vec<syn::WherePredicate>>,
    /// The parameters for `Hash`.
    hash: FieldHash,
    /// The parameters for `Eq`.
    partial_eq: FieldPartialEq,
}

#[derive(Debug, Default)]
/// Represent the `derivative(Clone(…))` attributes on an input.
pub struct InputClone {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
    /// Whether the implementation should have an explicit `clone_from`.
    pub clone_from: bool,
    /// Whether the `rustc_copy_clone_marker` was found.
    pub rustc_copy_clone_marker: bool,
}

#[derive(Debug, Default)]
/// Represent the `derivative(Clone(…))` attributes on an input.
pub struct InputCopy {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
    /// Wether the input also derive `Clone` (ie. `derive(Clone)`, but not `derivative(Clone)`)
    derives_clone: bool,
}

#[derive(Debug, Default)]
/// Represent the `derivative(Debug(…))` attributes on an input.
pub struct InputDebug {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
    /// Whether the type is marked `transparent`.
    pub transparent: bool,
}

#[derive(Debug, Default)]
/// Represent the `derivative(Default(…))` attributes on an input.
pub struct InputDefault {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
    /// Whether the type is marked with `new`.
    pub new: bool,
}

#[derive(Debug, Default)]
/// Represent the `derivative(Eq(…))` attributes on an input.
pub struct InputEq {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
}

#[derive(Debug, Default)]
/// Represent the `derivative(Hash(…))` attributes on an input.
pub struct InputHash {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
}

#[derive(Debug, Default)]
/// Represent the `derivative(PartialEq(…))` attributes on an input.
pub struct InputPartialEq {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
    /// Allow `derivative(PartialEq)` on enums:
    on_enum: bool,
}

#[derive(Debug, Default)]
/// Represents the `derivarive(Clone(…))` attributes on a field.
pub struct FieldClone {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
    /// The `clone_with` attribute if present and the path to the clonning function.
    clone_with: Option<syn::Path>,
}

#[derive(Debug, Default)]
/// Represents the `derivarive(Debug(…))` attributes on a field.
pub struct FieldDebug {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
    /// The `format_with` attribute if present and the path to the formatting function.
    format_with: Option<syn::Path>,
    /// Whether the field is to be ignored from output.
    ignore: bool,
}

#[derive(Debug, Default)]
/// Represent the `derivarive(Default(…))` attributes on a field.
pub struct FieldDefault {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
    /// The default value for the field if present.
    pub value: Option<syn::Expr>,
}

#[derive(Debug, Default)]
/// Represents the `derivarive(Hash(…))` attributes on a field.
pub struct FieldHash {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
    /// The `hash_with` attribute if present and the path to the hashing function.
    hash_with: Option<syn::Path>,
    /// Whether the field is to be ignored when hashing.
    ignore: bool,
}

#[derive(Debug, Default)]
/// Represent the `derivarive(PartialEq(…))` attributes on a field.
pub struct FieldPartialEq {
    /// The `bound` attribute if present and the corresponding bounds.
    bounds: Option<Vec<syn::WherePredicate>>,
    /// The `compare_with` attribute if present and the path to the comparison function.
    compare_with: Option<syn::Path>,
    /// Whether the field is to be ignored when comparing.
    ignore: bool,
}

macro_rules! for_all_attr {
    (for ($name:ident, $value:ident) in $attrs:expr; $($body:tt)*) => {
        for nested_metas in $attrs.into_iter().filter_map(|attr| derivative_attribute(&attr)) {
            for meta in nested_metas.into_iter().map(read_meta) {
                let Meta($name, $value) = meta?;
                match $name.as_ref() {
                    $($body)*
                    _ => return Err(format!("unknown trait `{}`", $name)),
                }
            }
        }
    };
}

macro_rules! match_attributes {
    (let Some($name:ident) = $unwrapped:expr; for $value:ident in $values:expr; $($body:tt)* ) => {
        let mut $name = $unwrapped.take().unwrap_or_default();

        match_attributes! {
            for $value in $values;
            $($body)*
        }

        $unwrapped = Some($name);
    };

    (for $value:ident in $values:expr; $($body:tt)* ) => {
        for (name, $value) in $values {
            match name.as_ref() {
                $($body)*
                _ => return Err(format!("unknown attribute `{}`", name)),
            }
        }
    };
}

impl Input {
    /// Parse the `derivative` attributes on a type.
    pub fn from_ast(attrs: &[syn::Attribute]) -> Result<Input, String> {
        let mut input = Input::default();

        for_all_attr! {
            for (name, values) in attrs;
            "Clone" => {
                let mut clone = input.clone.take().unwrap_or_default();

                clone.rustc_copy_clone_marker = attrs
                    .iter()
                    .any(|attr| {
                        let path = &attr.path;
                        let segments = &path.segments;

                        !path.global()
                            && segments.len() == 1
                            && segments[0].ident == "rustc_copy_clone_marker"
                            && segments[0].arguments == syn::PathArguments::None
                    });

                match_attributes! {
                    for value in values;
                    "bound" => parse_bound(&mut clone.bounds, value)?,
                    "clone_from" => {
                        clone.clone_from = parse_boolean_meta_item(value, true, "clone_from")?;
                    }
                }

                input.clone = Some(clone);
            }
            "Copy" => {
                let mut copy = input.copy.take().unwrap_or_default();

                for attr in attrs {
                    if let Some(syn::Meta::List(syn::MetaList { ident, nested, .. })) = attr.interpret_meta() {
                        fn is_clone(elem: &syn::NestedMeta) -> bool {
                            match *elem {
                                syn::NestedMeta::Meta(ref meta) => meta.name() == "Clone",
                                syn::NestedMeta::Literal(_) => false,
                            }
                        }

                        if ident == "derive" && nested.iter().any(is_clone) {
                            copy.derives_clone = true;
                        }
                    }
                }

                match_attributes! {
                    for value in values;
                    "bound" => parse_bound(&mut copy.bounds, value)?,
                }

                input.copy = Some(copy);
            }
            "Debug" => {
                match_attributes! {
                    let Some(debug) = input.debug;
                    for value in values;
                    "bound" => parse_bound(&mut debug.bounds, value)?,
                    "transparent" => {
                        debug.transparent = parse_boolean_meta_item(value, true, "transparent")?;
                    }
                }
            }
            "Default" => {
                match_attributes! {
                    let Some(default) = input.default;
                    for value in values;
                    "bound" => parse_bound(&mut default.bounds, value)?,
                    "new" => {
                        default.new = parse_boolean_meta_item(value, true, "new")?;
                    }
                }
            }
            "Eq" => {
                match_attributes! {
                    let Some(eq) = input.eq;
                    for value in values;
                    "bound" => parse_bound(&mut eq.bounds, value)?,
                }
            }
            "Hash" => {
                match_attributes! {
                    let Some(hash) = input.hash;
                    for value in values;
                    "bound" => parse_bound(&mut hash.bounds, value)?,
                }
            }
            "PartialEq" => {
                match_attributes! {
                    let Some(partial_eq) = input.partial_eq;
                    for value in values;
                    "bound" => parse_bound(&mut partial_eq.bounds, value)?,
                    "feature_allow_slow_enum" => {
                        partial_eq.on_enum = parse_boolean_meta_item(value, true, "feature_allow_slow_enum")?;
                    }
                }
            }
        }

        Ok(input)
    }

    pub fn clone_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.clone.as_ref().map_or(None, |d| d.bounds.as_ref().map(Vec::as_slice))
    }

    pub fn clone_from(&self) -> bool {
        self.clone.as_ref().map_or(false, |d| d.clone_from)
    }

    pub fn copy_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.copy.as_ref().map_or(None, |d| d.bounds.as_ref().map(Vec::as_slice))
    }

    pub fn derives_clone(&self) -> bool {
        self.copy.as_ref().map_or(false, |d| d.derives_clone)
    }

    pub fn debug_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.debug.as_ref().map_or(None, |d| d.bounds.as_ref().map(Vec::as_slice))
    }

    pub fn debug_transparent(&self) -> bool {
        self.debug.as_ref().map_or(false, |d| d.transparent)
    }

    pub fn default_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.default.as_ref().map_or(None, |d| d.bounds.as_ref().map(Vec::as_slice))
    }

    pub fn eq_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.eq.as_ref().map_or(None, |d| d.bounds.as_ref().map(Vec::as_slice))
    }

    pub fn hash_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.hash.as_ref().map_or(None, |d| d.bounds.as_ref().map(Vec::as_slice))
    }

    pub fn rustc_copy_clone_marker(&self) -> bool {
        self.clone.as_ref().map_or(false, |d| d.rustc_copy_clone_marker)
    }

    pub fn partial_eq_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.partial_eq.as_ref().map_or(None, |d| d.bounds.as_ref().map(Vec::as_slice))
    }

    pub fn partial_eq_on_enum(&self) -> bool {
        self.partial_eq.as_ref().map_or(false, |d| d.on_enum)
    }
}

impl Field {
    /// Parse the `derivative` attributes on a type.
    pub fn from_ast(field: &syn::Field) -> Result<Field, String> {
        let mut out = Field::default();

        for_all_attr! {
            for (name, values) in &field.attrs;
            "Clone" => {
                match_attributes! {
                    for value in values;
                    "bound" => parse_bound(&mut out.clone.bounds, value)?,
                    "clone_with" => {
                        let path = value.ok_or_else(|| "`clone_with` needs a value".to_string())?;
                        out.clone.clone_with = Some(syn::parse_str(&path).map_err(|e| e.to_string())?);
                    }
                }
            }
            "Debug" => {
                match_attributes! {
                    for value in values;
                    "bound" => parse_bound(&mut out.debug.bounds, value)?,
                    "format_with" => {
                        let path = value.ok_or_else(|| "`format_with` needs a value".to_string())?;
                        out.debug.format_with = Some(syn::parse_str(&path).map_err(|e| e.to_string())?);
                    }
                    "ignore" => {
                        out.debug.ignore = parse_boolean_meta_item(value, true, "ignore")?;
                    }
                }
            }
            "Default" => {
                match_attributes! {
                    for value in values;
                    "bound" => parse_bound(&mut out.default.bounds, value)?,
                    "value" => {
                        let value = value.ok_or_else(|| "`value` needs a value".to_string())?;
                        out.default.value = Some(syn::parse_str(&value).map_err(|e| e.to_string())?);
                    }
                }
            }
            "Eq" => {
                match_attributes! {
                    for value in values;
                    "bound" => parse_bound(&mut out.eq_bound, value)?,
                }
            }
            "Hash" => {
                match_attributes! {
                    for value in values;
                    "bound" => parse_bound(&mut out.hash.bounds, value)?,
                    "hash_with" => {
                        let path = value.ok_or_else(|| "`hash_with` needs a value".to_string())?;
                        out.hash.hash_with = Some(syn::parse_str(&path).map_err(|e| e.to_string())?);
                    }
                    "ignore" => {
                        out.hash.ignore = parse_boolean_meta_item(value, true, "ignore")?;
                    }
                }
            }
            "PartialEq" => {
                match_attributes! {
                    for value in values;
                    "bound" => parse_bound(&mut out.partial_eq.bounds, value)?,
                    "compare_with" => {
                        let path = value.ok_or_else(|| "`compare_with` needs a value".to_string())?;
                        out.partial_eq.compare_with = Some(syn::parse_str(&path).map_err(|e| e.to_string())?);
                    }
                    "ignore" => {
                        out.partial_eq.ignore = parse_boolean_meta_item(value, true, "ignore")?;
                    }
                }
            }
        }

        Ok(out)
    }

    pub fn clone_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.clone.bounds.as_ref().map(Vec::as_slice)
    }

    pub fn clone_with(&self) -> Option<&syn::Path> {
        self.clone.clone_with.as_ref()
    }

    pub fn copy_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.copy_bound.as_ref().map(Vec::as_slice)
    }

    pub fn debug_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.debug.bounds.as_ref().map(Vec::as_slice)
    }

    pub fn debug_format_with(&self) -> Option<&syn::Path> {
        self.debug.format_with.as_ref()
    }

    pub fn ignore_debug(&self) -> bool {
        self.debug.ignore
    }

    pub fn ignore_hash(&self) -> bool {
        self.hash.ignore
    }

    pub fn default_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.default.bounds.as_ref().map(Vec::as_slice)
    }

    pub fn default_value(&self) -> Option<&syn::Expr> {
        self.default.value.as_ref()
    }

    pub fn eq_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.eq_bound.as_ref().map(Vec::as_slice)
    }

    pub fn hash_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.hash.bounds.as_ref().map(Vec::as_slice)
    }

    pub fn hash_with(&self) -> Option<&syn::Path> {
        self.hash.hash_with.as_ref()
    }

    pub fn partial_eq_bound(&self) -> Option<&[syn::WherePredicate]> {
        self.partial_eq.bounds.as_ref().map(Vec::as_slice)
    }

    pub fn partial_eq_compare_with(&self) -> Option<&syn::Path> {
        self.partial_eq.compare_with.as_ref()
    }

    pub fn ignore_partial_eq(&self) -> bool {
        self.partial_eq.ignore
    }
}

/// Filter the `derivative` items from an attribute.
fn derivative_attribute(
    attr: &syn::Attribute,
) -> Option<syn::punctuated::Punctuated<syn::NestedMeta, Token![,]>> {
    attr.interpret_meta().and_then(|meta| {
        match meta {
            syn::Meta::List(syn::MetaList { ident, nested, .. }) => {
                if ident == "derivative" {
                    Some(nested)
                } else {
                    None
                }
            }
            syn::Meta::Word(..) |
            syn::Meta::NameValue(..) => None,
        }
    })
}

/// Represent an attribute.
///
/// We only have a limited set of possible attributes:
///
/// * `#[derivative(Debug)]` is represented as `("Debug", [])`;
/// * `#[derivative(Debug="foo")]` is represented as `("Debug", [("foo", None)])`;
/// * `#[derivative(Debug(foo="bar")]` is represented as `("Debug", [("foo", Some("bar"))])`.
struct Meta(syn::Ident, Vec<(syn::Ident, Option<String>)>);

/// Parse an arbitrary meta for our limited `Meta` subset.
fn read_meta(meta: syn::NestedMeta) -> Result<Meta, String> {
    let meta = match meta {
        syn::NestedMeta::Meta(meta) => meta,
        syn::NestedMeta::Literal(_) => {
            return Err("Expected meta but found literal".to_string());
        }
    };

    match meta {
        syn::Meta::Word(name) => Ok(Meta(name, Vec::new())),
        syn::Meta::List(syn::MetaList { ident, nested, .. }) => {
            let values = nested
                .into_iter()
                .map(|value| {
                    if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue { ident, lit, .. })) = value {
                        let string_lit = string_or_err(&lit)?;

                        Ok((ident, Some(string_lit)))
                    } else {
                        Err("Expected named value".to_string())
                    }
                })
                .collect::<Result<_, _>>()?;

            Ok(Meta(ident, values))
        }
        syn::Meta::NameValue(syn::MetaNameValue { ident, lit, .. }) => {
            let string_lit = string_or_err(&lit)?;

            Ok(Meta(ident, vec![(syn::Ident::from(string_lit), None)]))
        }
    }
}

/// Parse a `bound` item.
fn parse_bound(
    opt_bounds: &mut Option<Vec<syn::WherePredicate>>,
    value: Option<String>,
) -> Result<(), String> {
    let mut bounds = opt_bounds.take().unwrap_or_default();
    let bound = value.ok_or_else(|| "`bound` needs a value".to_string())?;

    if !bound.is_empty() {
        let where_clause: syn::WhereClause = syn::parse_str(&format!("where {}", bound)).map_err(|e| e.to_string())?;
        bounds.extend(where_clause.predicates);
    }

    *opt_bounds = Some(bounds);

    Ok(())
}

/// Parse an item value as a boolean. Accepted values are the string literal `"true"` and
/// `"false"`. The `default` parameter specifies what the value of the boolean is when only its
/// name is specified (eg. `Debug="ignore"` is equivalent to `Debug(ignore="true")`). The `name`
/// parameter is used for error reporting.
fn parse_boolean_meta_item(item: Option<String>, default: bool, name: &str) -> Result<bool, String> {
    match item.as_ref().map(String::as_str) {
        Some("true") => Ok(true),
        Some("false") => Ok(false),
        Some(_) => Err(format!("Invalid value for `{}`", name)),
        None => Ok(default),
    }
}

/// Get the string out of a string literal or report an error for other literals.
fn string_or_err(lit: &syn::Lit) -> Result<String, String> {
    if let syn::Lit::Str(ref lit_str) = *lit {
        Ok(lit_str.value())
    } else {
        Err("Expected string".to_string())
    }
}
