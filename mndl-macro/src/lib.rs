mod expr;
mod item;

use std::str::FromStr;

use expr::*;
use item::*;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse_macro_input;

/// procedural macros that generates a `struct` for parametric drawing
///
/// the produced struct conforms to [`mandala::VectorValuedFn`]
///
/// the [`encase::ShaderType`] is derived for the given struct
///
/// ## Example
///
/// ```
/// use std::f32::consts::PI;
///
/// use mndl_macro::valued_struct;
/// use mandala::VectorValuedFn;
///
/// valued_struct!{
///     struct Circle {
///         center: [f32; 3],
///         radii: [f32; 2]
///     },
///     x(t) -> self.center[0] + self.radii[0] * (t * PI * 2.0).cos(),
///     y(t) -> self.center[1] + self.radii[1] * (t * PI * 2.0).sin(),
///     z(_) -> self.center[2]
/// }
///
/// let example = Circle {
///     center: [0.0, 0.0, 0.0],
///     radii: [20.0, 20.0]
/// };
///
/// let sample = example.eval(1.0);
/// assert_eq!(sample.x, 20.0);
/// ```
#[proc_macro]
pub fn valued_struct(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(tokens as ValuedItem);
    let ValuedItem {
        param_struct,
        x_fn,
        y_fn,
        z_fn,
    } = item.clone();

    let params_ident = param_struct.ident.clone();

    let fields = [x_fn, y_fn, z_fn];

    let eval_methods = fields.iter().filter_map(|f| {
        f.as_ref().map(|field| {
            let ax = field.axis.clone();
            let p = field
                .param
                .clone()
                .map(|i| i.to_token_stream())
                .unwrap_or(TokenStream::from_str("_").unwrap());
            let expr = field.expr.clone();
            quote! {
                pub fn #ax(&self, #p: mandala::Float) -> mandala::Float {
                    #expr
                }
            }
        })
    });

    let eval_fields = fields.iter().filter_map(|f| {
        f.as_ref().map(|field| {
            let ax = field.axis.clone();
            quote! {
                #ax: self.#ax(t),
            }
        })
    });

    let shdr = item.shader_code();

    let expanded = quote! {
        #param_struct

        impl #params_ident {
            #(#eval_methods)*
        }

        impl mandala::VectorValuedFn for #params_ident {
            fn eval(&self, t: mandala::Float) -> mandala::Vector {
                mandala::Vector {
                    #(#eval_fields)*
                }
            }

            fn to_shader_code(&self) -> naga::Module {
                let shdr = #shdr;

                naga::front::wgsl::parse_str(shdr).expect("generated invalid module")
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
