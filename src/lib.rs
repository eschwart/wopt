use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Field, Fields, Ident, Meta, Type, parse_macro_input, punctuated::Iter};

#[cfg(all(feature = "rkyv", feature = "serde"))]
compile_error!("Features `rkyv` and `serde` cannot be enabled at the same time!");

// function to extract `#[wopt(derive(...))]`
fn extract_derive_traits(input: &DeriveInput) -> Vec<Ident> {
    let mut traits = Vec::new();

    for attr in &input.attrs {
        if attr.path().is_ident("wopt") {
            let meta = attr.parse_args::<Meta>().unwrap();
            let meta_list = meta.require_list().unwrap();

            meta_list
                .parse_nested_meta(|a| {
                    if let Some(ident) = a.path.get_ident() {
                        traits.push(ident.clone());
                    }
                    Ok(())
                })
                .unwrap();
        }
    }
    traits
}

fn get_field_kvs(
    fields: Iter<Field>,
    f: fn(&Option<Ident>, &Type, bool) -> proc_macro2::TokenStream,
) -> Vec<proc_macro2::TokenStream> {
    fields
        .map(|field: &Field| {
            if field.attrs.len() > 1 {
                panic!("Only 1 attribute per field is supported.")
            }
            let mut is_required = false;

            if let Some(attr) = field.attrs.first() {
                if attr.path().is_ident("wopt") {
                    let mut n = 0;
                    attr.parse_nested_meta(|a| {
                        if a.path.is_ident("required") {
                            is_required = true
                        }
                        n += 1;
                        Ok(())
                    })
                    .unwrap();

                    if n > 1 {
                        panic!("A field has too many `wopt` attr args (max: 1)")
                    }
                }
            }
            let (field_name, field_type) = (&field.ident, &field.ty);
            f(field_name, field_type, is_required)
        })
        .collect()
}

#[proc_macro_derive(WithOpt, attributes(wopt))]
pub fn wopt_derive(input: TokenStream) -> TokenStream {
    // parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // get the struct name and generate the Opt name
    let name = &input.ident;
    let opt_name = Ident::new(&format!("{}Opt", name), name.span());

    // extract custom `#[wopt(derive(...))]` attributes
    let derived_traits = extract_derive_traits(&input);

    // the type of struct
    let mut is_named = false;

    // match on the fields of the struct
    let fields: Vec<_> = if let syn::Data::Struct(ref data) = input.data {
        match &data.fields {
            Fields::Named(fields) => {
                is_named = true;
                get_field_kvs(
                    fields.named.iter(),
                    |field_name: &Option<Ident>, field_type: &Type, is_required: bool| {
                        if is_required {
                            quote! { pub #field_name: #field_type, }
                        } else {
                            quote! { pub #field_name: Option<#field_type>, }
                        }
                    },
                )
            }
            Fields::Unnamed(fields) => get_field_kvs(
                fields.unnamed.iter(),
                |_: &Option<Ident>, field_type: &Type, is_required: bool| {
                    if is_required {
                        quote! { pub #field_type, }
                    } else {
                        quote! { pub Option<#field_type>, }
                    }
                },
            ),
            _ => panic!("Unit structs are not supported."),
        }
    } else {
        panic!("Only structs are supported");
    };

    let derives = if derived_traits.is_empty() {
        quote! {}
    } else {
        quote! { #(#derived_traits),* }
    };

    #[cfg(feature = "rkyv")]
    let derives = quote! { #derives, ::rkyv::Archive, ::rkyv::Deserialize, ::rkyv::Serialize };

    #[cfg(feature = "serde")]
    let derives = quote! { #derives, ::serde::Deserialize, ::serde::Serialize };

    // generate the new struct
    let expanded = if is_named {
        quote! {
            #[derive(#derives)]
            pub struct #opt_name {
                #(#fields)*
            }
        }
    } else {
        quote! {
            #[derive(#derives)]
            pub struct #opt_name(#(#fields)*);
        }
    };

    // convert into TokenStream
    expanded.into()
}
