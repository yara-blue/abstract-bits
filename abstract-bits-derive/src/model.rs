use proc_macro_error2::{OptionExt, abort};
use proc_macro2::{Span, TokenStream, TokenTree};
use syn::parse_quote_spanned;
use syn::spanned::Spanned;
use syn::{Attribute, GenericArgument, Ident, PathArguments, Visibility};

#[derive(Debug)]
pub struct Model {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub ident: Ident,
    pub ty: Type,
}

#[derive(Debug)]
pub struct EmptyVariant {
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
    pub discriminant: usize,
}

#[derive(Debug)]
pub enum Type {
    NormalStruct(Vec<Field>),
    UnitStruct(syn::Field),
    Enum {
        bits: usize,
        variants: Vec<EmptyVariant>,
        // Extracted as Ident from parsed AST, no reason to change that
        repr_type: Ident,
    },
}

#[derive(Debug, Clone)]
pub struct NormalField {
    pub vis: Visibility,
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
    pub out_ty: syn::Type,
    pub bits: Option<u8>,
}

fn out_ty_from_padding(padding: u8, span: Span) -> syn::Type {
    match padding {
        1..=8 => parse_quote_spanned!(span =>u8),
        9..=16 => parse_quote_spanned!(span =>u16),
        17..=32 => parse_quote_spanned!(span =>u32),
        33..=64 => parse_quote_spanned!(span =>u64),
        _other => abort!(span, "unsupported field size"),
    }
}

impl NormalField {
    fn from(field: syn::Field) -> Self {
        let mut bits = None;
        let mut out_ty = field.ty.clone();
        if let Ok(padding) = padding_from_type(&field.ty) {
            if padding != 8 && padding != 16 && padding != 32 && padding != 64 {
                out_ty = out_ty_from_padding(padding, field.ty.span());
                bits = Some(padding);
            }
        }

        NormalField {
            vis: field.vis,
            attrs: field.attrs,
            ident: field.ident.expect("unit struct not handled by NormalField"),
            out_ty,
            bits,
        }
    }
}

#[derive(Debug)]
pub enum Field {
    Normal(NormalField),
    Option {
        full_type: NormalField,
        inner_type: NormalField,
        controller: syn::Expr,
    },
    List {
        full_type: NormalField,
        inner_type: NormalField,
        max_len: usize,
        controller: syn::Expr,
    },
    Array {
        length: syn::Expr,
        inner_type: syn::Type,
        field: syn::Field,
    },
    HiddenController {
        field: NormalField,
        controlled: Ident,
        presence: bool,
    },
    PaddBits(u8),
}

impl Field {
    pub fn needed_in_struct_def(&self) -> Option<NormalField> {
        match self {
            Field::Normal(field)
            | Field::Option {
                full_type: field, ..
            }
            | Field::List {
                full_type: field, ..
            } => {
                let mut filtered_field = field.clone();
                filtered_field.attrs = filtered_field
                    .attrs
                    .into_iter()
                    .filter(|attr| !attr.path().is_ident("abstract_bits"))
                    .collect();
                Some(filtered_field)
            }
            Field::Array { field, .. } => Some(NormalField {
                vis: field.vis.clone(),
                attrs: field
                    .attrs
                    .clone()
                    .into_iter()
                    .filter(|attr| !attr.path().is_ident("abstract_bits"))
                    .collect(),
                ident: field
                    .ident
                    .clone()
                    .expect("code is not run for unit structs"),
                out_ty: field.ty.clone(),
                bits: None,
            }),
            _ => None,
        }
    }
}

fn padding_from_type(ty: &syn::Type) -> Result<u8, (&'static str, Span)> {
    let syn::Type::Path(ty) = ty else {
        abort!(ty.span(), "only normal types are supported");
    };

    let end = ty.path.segments.last().expect("type can not be empty");
    match end.ident.to_string().trim_start_matches("u").parse() {
        Ok(padding) => Ok(padding),
        Err(_) => Err((
            "field did not start with u and/or did not end in number",
            end.ident.span(),
        )),
    }
}

impl Field {
    fn from(field: syn::Field, previous_fields: &[Field]) -> Self {
        let ident = field
            .ident
            .as_ref()
            .expect("unit structs are not tranformed into model::Field");
        if ident == "reserved" {
            let padding = padding_from_type(&field.ty)
                .unwrap_or_else(|(msg, span)| abort!(span, msg));
            Self::PaddBits(padding)
        } else if let Some(option_stripped) = strip_option(field.clone()) {
            let controller = presence_from_attr(&field).unwrap_or_else(|| {
                abort!(
                    ident.span(),
                    "Option field '{}' requires presence_from attribute",
                    ident
                )
            });
            Self::Option {
                inner_type: NormalField::from(option_stripped),
                full_type: NormalField::from(field),
                controller,
            }
        } else if let Some(vec_stripped) = strip_vec(field.clone()) {
            let controller = length_from_attr(&field).unwrap_or_else(|| {
                abort!(
                    ident.span(),
                    "List field '{}' requires length_from attribute",
                    ident
                )
            });
            let max_len = max_size_from_controller_field(&controller, previous_fields);
            Self::List {
                inner_type: NormalField::from(vec_stripped),
                max_len,
                full_type: NormalField::from(field),
                controller,
            }
        } else if let syn::Type::Array(a) = &field.ty {
            Self::Array {
                inner_type: *a.elem.clone(),
                length: a.len.clone(),
                field,
            }
        } else {
            Self::Normal(NormalField::from(field))
        }
    }
}

fn extract_expr_field_base_path(field_expr: &syn::ExprField) -> Option<&syn::ExprPath> {
    if let syn::Expr::Path(base_path) = &*field_expr.base {
        Some(base_path)
    } else {
        None
    }
}

fn controller_in_previous_fields<'a>(
    previous_fields: &'a [Field],
    controller_ident: &Ident,
) -> Option<&'a NormalField> {
    previous_fields.iter().find_map(|f| match f {
        Field::Normal(nf) if nf.ident == *controller_ident => Some(nf),
        _ => None,
    })
}

fn max_size_from_controller_field(
    controller_expr: &syn::Expr,
    previous_fields: &[Field],
) -> usize {
    // Extract the base field name from the expression
    let base_path = match controller_expr {
        syn::Expr::Path(path) => path,
        syn::Expr::Field(expr) if let syn::Expr::Path(path) = &*expr.base => path,
        _ => abort!(
            controller_expr.span(),
            "Controller expression must be a field name or field access"
        ),
    };

    if base_path.path.segments.len() != 1 {
        abort!(
            controller_expr.span(),
            "Complex controller expressions not yet supported"
        );
    }

    let controller_ident = &base_path.path.segments[0].ident;

    // Look for the controller field in previous_fields
    let ident = controller_in_previous_fields(previous_fields, controller_ident)
        .unwrap_or_else(|| {
            abort!(
                controller_ident.span(),
                "Controller field '{}' not found",
                controller_ident
            )
        });

    // Compute the size
    if let Some(bits) = ident.bits {
        2usize.pow(bits as u32)
    } else {
        if let Ok(bits) = padding_from_type(&ident.out_ty) {
            2usize.pow(bits as u32)
        } else {
            abort!(
                controller_ident.span(),
                "Controller field '{}' must be a numeric type with known bit size",
                controller_ident
            );
        }
    }
}

fn strip_vec(field: syn::Field) -> Option<syn::Field> {
    strip_generic(field, "Vec")
}

fn strip_option(field: syn::Field) -> Option<syn::Field> {
    strip_generic(field, "Option")
}

fn strip_generic(field: syn::Field, outer_ident: &str) -> Option<syn::Field> {
    let syn::Type::Path(path) = &field.ty else {
        return None;
    };

    let ty = &path.path.segments.first()?;
    if ty.ident != outer_ident {
        return None;
    }

    let PathArguments::AngleBracketed(generics) = &ty.arguments else {
        return None;
    };

    let Some(GenericArgument::Type(inner_type)) = generics.args.first() else {
        return None;
    };

    let mut new_field = field.clone();
    new_field.ty = inner_type.clone();
    Some(new_field)
}

fn length_from_attr(field: &syn::Field) -> Option<syn::Expr> {
    fn parse(attr: &Attribute) -> Option<Result<syn::Expr, ()>> {
        let Ok(list) = attr.meta.require_list() else {
            return Some(Err(()));
        };
        let mut tokens = list.tokens.clone().into_iter();
        match tokens.next() {
            Some(TokenTree::Ident(ident)) if ident == "length_from" => (),
            _ => return None,
        }
        match tokens.next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == '=' => (),
            _ => return Some(Err(())),
        }

        // Parse remaining tokens as expression
        Some(syn::parse2::<syn::Expr>(tokens.collect()).map_err(|_| ()))
    }

    let attr = field
        .attrs
        .iter()
        .find(|a| a.path().is_ident("abstract_bits"))?;

    match parse(attr)? {
        Ok(expr) => Some(expr),
        Err(_) => abort!(attr.span(), "invalid abstract_bits attribute"; 
            help = "The syntax is: #[abstract_bits(length_from = <expr>)] with expr \
            an expression controlling this vec's length"),
    }
}

fn presence_from_attr(field: &syn::Field) -> Option<syn::Expr> {
    fn parse(attr: &Attribute) -> Option<Result<syn::Expr, ()>> {
        let Ok(list) = attr.meta.require_list() else {
            return Some(Err(()));
        };
        let mut tokens = list.tokens.clone().into_iter();
        match tokens.next() {
            Some(TokenTree::Ident(ident)) if ident == "presence_from" => (),
            _ => return None,
        }
        match tokens.next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == '=' => (),
            _ => return Some(Err(())),
        }

        // Parse remaining tokens as expression
        Some(syn::parse2::<syn::Expr>(tokens.collect()).map_err(|_| ()))
    }

    let attr = field
        .attrs
        .iter()
        .find(|a| a.path().is_ident("abstract_bits"))?;

    match parse(attr)? {
        Ok(expr) => Some(expr),
        Err(_) => abort!(attr.span(), "invalid abstract_bits attribute"; 
            help = "The syntax is: #[abstract_bits(presence_from = <expr>)] where the \
            boolean expr controls this option's presence"),
    }
}

impl Model {
    fn reject_item_generics(generics: &syn::Generics) {
        assert!(generics.lifetimes().count() == 0, "lifetimes not supported");
        assert!(
            generics.const_params().count() == 0,
            "const params not supported"
        );
        assert!(
            generics.type_params().count() == 0,
            "generic types not supported"
        );
    }

    pub(crate) fn from_enum(item: syn::ItemEnum, attr: TokenStream) -> Self {
        let Ok(bits) = get_num_bits(attr) else {
            abort!(item.span(), "Every enum must be attributed with its serialized size \
                in bits."; note = "Example: #[abstract_bits::abstract_bits(bits=2)]");
        };
        Self::reject_item_generics(&item.generics);

        let repr = require_repr_attr(&item.attrs, item.span());
        let variants: Vec<_> = item
            .variants
            .clone()
            .into_iter()
            .map(|v| EmptyVariant {
                attrs: v.attrs,
                ident: v.ident,
                discriminant: require_usize(
                    v.discriminant
                        .clone()
                        .unwrap_or_else(|| {
                            abort!(item.span(), "Every enum variant must have an explicit \
                    discriminant value"; 
                    note = "Assign a discriminant with = <number>")
                        })
                        .1,
                ),
            })
            .collect();
        verify_all_discriminants_fit(&variants, bits);

        let ty = Type::Enum {
            bits,
            variants,
            repr_type: repr,
        };

        Self {
            attrs: item.attrs,
            vis: item.vis,
            ident: item.ident,
            ty,
        }
    }
    pub(crate) fn from_struct(item: syn::ItemStruct, _attr: TokenStream) -> Self {
        Self::reject_item_generics(&item.generics);

        let is_unit = item
            .fields
            .iter()
            .next()
            .expect_or_abort("structs without fields are not supported")
            .ident
            .is_none();
        let ty = if is_unit {
            let field = item.fields.clone().into_iter().next().unwrap_or_else(|| {
                abort!(item.span(), "Zero sized struct not supported")
            });
            Type::UnitStruct(field)
        } else {
            let mut fields = Vec::new();
            for item in item.fields {
                let field = Field::from(item, &fields);
                fields.push(field);
            }
            hide_same_struct_controllers(&mut fields);
            Type::NormalStruct(fields)
        };

        Self {
            attrs: item.attrs,
            vis: item.vis,
            ident: item.ident,
            ty,
        }
    }
}

// A controller field on the same struct is turned into a `HiddenController` to allow its
// value to be set automatically based on the presence of the controlled field. Otherwise,
// the state of both fields needs to be manually aligned.
fn hide_same_struct_controllers(fields: &mut [Field]) {
    let mut controllers: Vec<(Ident, Ident, bool)> = Vec::new();
    for field in fields.iter() {
        let (controller, controlled, presence) = match field {
            Field::Option {
                controller,
                full_type,
                ..
            } => (controller, &full_type.ident, true),
            Field::List {
                controller,
                full_type,
                ..
            } => (controller, &full_type.ident, false),
            _ => continue,
        };
        if let Some(ident) = same_struct_controller(controller) {
            controllers.push((ident.clone(), controlled.clone(), presence));
        }
    }

    for field in fields.iter_mut() {
        let Field::Normal(normal) = field else {
            continue;
        };
        if let Some((_, controlled, presence)) = controllers
            .iter()
            .find(|(ident, ..)| *ident == normal.ident)
        {
            *field = Field::HiddenController {
                field: normal.clone(),
                controlled: controlled.clone(),
                presence: *presence,
            };
        }
    }
}

fn same_struct_controller(expr: &syn::Expr) -> Option<&Ident> {
    match expr {
        syn::Expr::Path(path) if path.path.segments.len() == 1 => {
            Some(&path.path.segments[0].ident)
        }
        _ => None,
    }
}

fn verify_all_discriminants_fit(variants: &[EmptyVariant], bits: usize) {
    let biggest = variants
        .iter()
        .max_by_key(|var| var.discriminant)
        .expect("zero size enums are not supported");
    if biggest.discriminant >= 2usize.pow(bits as u32) {
        abort!(
            biggest.ident.span(),
            "The discriminant for {} does not fit into {} bits",
            biggest.ident,
            bits
        );
    }
}

fn get_num_bits(attr: TokenStream) -> Result<usize, ()> {
    let mut tokens = attr.into_iter();
    match tokens.next() {
        Some(TokenTree::Ident(item)) if item == "bits" => (),
        _ => return Err(()),
    }

    match tokens.next() {
        Some(TokenTree::Punct(punct)) if punct.as_char() == '=' => (),
        _ => return Err(()),
    }

    let Some(TokenTree::Literal(num)) = tokens.next() else {
        return Err(());
    };

    num.to_string().parse().map_err(|_| ())
}

fn require_repr_attr(attrs: &[Attribute], span: Span) -> Ident {
    let attr = attrs
        .iter()
        .find(|a| a.path().is_ident("repr"))
        .unwrap_or_else(|| abort!(span, "enum must have repr attribute"));

    let list = attr
        .meta
        .require_list()
        .expect("we just found an attribute therefore its non empty");

    let Some(TokenTree::Ident(repr_type)) = list.tokens.clone().into_iter().next() else {
        abort!(span, "repr attribute on enum should contain repr type");
    };

    repr_type
}

fn require_usize(expr: syn::Expr) -> usize {
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(d),
        ..
    }) = expr
    {
        d.base10_parse()
            .expect("only valid numbers can be enum discriminant")
    } else {
        unreachable!("only digits form a valid enum discriminant expression")
    }
}
