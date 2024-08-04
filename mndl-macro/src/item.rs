use proc_macro2::Span;
use quote::ToTokens;
use syn::{parse::Parse, spanned::Spanned, ItemStruct, Lit, PatLit, Token};

use super::ValueFieldExpr;

pub const NAME_PREFIX: &str = "MNDL_Valued_";

#[derive(Clone)]
pub struct ValuedItem {
    pub param_struct: ItemStruct,
    pub x_fn: Option<ValueFieldExpr>,
    pub y_fn: Option<ValueFieldExpr>,
    pub z_fn: Option<ValueFieldExpr>,
}

impl Parse for ValuedItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let param_struct = input.parse::<ItemStruct>()?;
        input.parse::<Token![,]>()?;

        let mut x_fn = None;
        let mut y_fn = None;
        let mut z_fn = None;

        while !input.is_empty() {
            let field = input.parse::<ValueFieldExpr>()?;

            match field.axis.to_string().as_str() {
                "x" => x_fn = Some(field),
                "y" => y_fn = Some(field),
                "z" => z_fn = Some(field),
                _ => return Err(syn::Error::new(field.axis.span(), "Invalid axis")),
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(ValuedItem {
            param_struct,
            x_fn,
            y_fn,
            z_fn,
        })
    }
}

fn to_shader_type(ty: &syn::Type, span: Span) -> syn::Result<String> {
    match ty {
        syn::Type::Array(arr) => {
            let size = match &arr.len {
                syn::Expr::Lit(syn::ExprLit {
                    lit: Lit::Int(int), ..
                }) => int.to_string(),
                _ => {
                    return Err(syn::Error::new(
                        span,
                        "dynamically sized arrays not supported",
                    ))
                }
            };

            let ty = to_shader_type(&arr.elem, arr.span())?;

            Ok(format!("array<{ty}, {size}>"))
        }
        syn::Type::Path(p) => match p.path.to_token_stream().to_string().as_str() {
            "u32" | "i32" | "f32" | "bool" => Ok(p.path.to_token_stream().to_string()),
            "Vec3" | "glam::Vec3" | t => Err(syn::Error::new(
                p.span(),
                format!("unsupported type: {}", t).as_str(),
            )),
        },
        syn::Type::Tuple(_) => todo!(),
        _ => Err(syn::Error::new(
            span,
            format!("unsupported type: {}", ty.to_token_stream().to_string()).as_str(),
        )),
    }
}

impl ValuedItem {
    pub fn shader_code(&self) -> syn::Result<String> {
        let id = format!("{NAME_PREFIX}{}", self.param_struct.ident.to_string());

        let mut fields = String::default();

        for fld in self.param_struct.fields.iter() {
            let ty = to_shader_type(&fld.ty, fld.span())?;

            let name = fld.ident.as_ref().ok_or(syn::Error::new(
                fld.span(),
                "Only structs with named fields are supported",
            ))?;

            fields.extend(
                format!(
                    r#",
                    {name}: {ty}"#,
                )
                .chars(),
            )
        }

        Ok(format!(
            r#"
struct {id} {{
    {fields}
}}
        "#,
        ))
    }
}
