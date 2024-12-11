use darling::ast::{Data, Style};
use darling::{FromDeriveInput, FromField};
use quote::quote;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(error_info))]
struct StructData {
    ident: syn::Ident,
    generics: syn::Generics,
    data: Data<(), StructFields>,
}

#[derive(Debug, FromField)]
struct StructFields {
    ident: Option<syn::Ident>,
    ty: syn::Type,
}

pub(crate) fn process_from_js(input: syn::DeriveInput) -> proc_macro2::TokenStream {
    let (ident, generics, fields, merged) = parse_struct(input);

    let code = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().expect("Field name is missing");
        let field_ty = &field.ty;
        quote! {
            let #field_name:#field_ty = obj.get(stringify!(#field_name))?;
        }
    });
    let idents = fields.iter().map(|field| {
        let name = field.ident.as_ref().expect("Field name is missing");
        quote! {
            #name,
        }
    });

    quote! {
        impl #merged rquickjs::FromJs<'js> for #ident #generics {
            fn from_js(_ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
                let obj = value.into_object().unwrap();
                #(#code)*
                Ok(#ident {
                    #(#idents)*
                })
            }
        }
    }
}

pub(crate) fn process_into_js(input: syn::DeriveInput) -> proc_macro2::TokenStream {
    let (ident, generics, fields, merged) = parse_struct(input);
    let code = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().expect("Field name is missing");
        let _field_ty = &field.ty;
        quote! {
            obj.set(stringify!(#field_name), self.#field_name)?;
        }
    });
    let _idents = fields.iter().map(|field| {
        let name = field.ident.as_ref().expect("Field name is missing");
        quote! {
            #name,
        }
    });

    quote! {
        impl #merged rquickjs::IntoJs<'js> for #ident #generics {
            fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
                let obj = rquickjs::Object::new(ctx.clone())?;
                #(#code)*
                Ok(obj.into_value())
            }
        }
    }
}

fn parse_struct(
    input: syn::DeriveInput,
) -> (syn::Ident, syn::Generics, Vec<StructFields>, syn::Generics) {
    let StructData {
        ident,
        generics,
        data: Data::Struct(fields),
    } = StructData::from_derive_input(&input).expect("Failed to parse input")
    else {
        panic!("Only structs are supported")
    };

    let fields = match fields.style {
        Style::Struct => fields.fields,
        _ => panic!("Only named fields are supported"),
    };
    let mut merged = generics.clone();
    merged.params.push(syn::parse_quote!('js));
    (ident, generics, fields, merged)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_from_js() {
        let input = r#"
            #[derive(FromJs)]
            struct Request {
                method: String,
                url: String,
                headers: HashMap<String, String>,
                body: Option<String>,
            }
        "#;
        let parse = syn::parse_str(input).unwrap();
        let info = StructData::from_derive_input(&parse).unwrap();
        assert_eq!(info.ident.to_string(), "Request");
        let output = process_from_js(parse);
        // let expected = quote! {
        //     impl<'js> rquickjs::FromJs<'js> for Request {
        //         fn from_js(_ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        //             let obj = value.into_object().unwrap();
        //             let method: String = obj.get("method")?;
        //             let url: String = obj.get("url")?;
        //             let headers: HashMap<String, String> = obj.get("headers")?;
        //             let body: Option<String> = obj.get("body")?;
        //             Ok(Request {
        //                 method,
        //                 url,
        //                 headers,
        //                 body,
        //             })
        //         }
        //     }
        // };
        println!("{}", output);
    }

    #[test]
    fn test_process_into_js() {
        let input = r#"
            #[derive(IntoJs)]
            struct Request {
                method: String,
                url: String,
                headers: HashMap<String, String>,
                body: Option<String>,
            }
        "#;
        let parse = syn::parse_str(input).unwrap();
        let info = StructData::from_derive_input(&parse).unwrap();
        assert_eq!(info.ident.to_string(), "Request");
        let output = process_into_js(parse);

        println!("{}", output);
    }
}
