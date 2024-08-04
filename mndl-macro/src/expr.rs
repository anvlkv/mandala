use syn::{parenthesized, parse::Parse, Expr, Ident, Token};

#[derive(Clone)]
pub struct ValueFieldExpr {
    pub axis: Ident,
    pub param: Option<Ident>,
    pub expr: Expr,
}

impl Parse for ValueFieldExpr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let axis = input.parse::<Ident>()?;
        let content;
        parenthesized!(content in input);
        let param = content.parse::<Ident>().ok();
        if param.is_none() {
            _ = content.parse::<Token![_]>()?;
        }
        input.parse::<Token![->]>()?;
        let expr = input.parse::<Expr>()?;

        Ok(Self { axis, param, expr })
    }
}
