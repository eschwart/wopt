use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Fields, Ident, Meta, parse_macro_input};

#[cfg(all(feature = "rkyv", feature = "serde"))]
compile_error!("Features `rkyv` and `serde` cannot be enabled at the same time!");

// Function to extract `#[wopt(derive(...))]`
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

#[proc_macro_derive(WithOpt, attributes(wopt))]
pub fn wopt_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the struct name and generate the Opt name
    let name = &input.ident;
    let opt_name = Ident::new(&format!("{}Opt", name), name.span());

    // Extract custom `#[wopt(derive(...))]` attributes
    let derived_traits = extract_derive_traits(&input);

    // Match on the fields of the struct
    let fields = if let syn::Data::Struct(ref data) = input.data {
        if let Fields::Named(ref fields) = data.fields {
            fields
                .named
                .iter()
                .map(|field| {
                    let field_name = &field.ident;
                    let field_type = &field.ty;

                    // Wrap each field's type with Option
                    quote! {
                        pub #field_name: Option<#field_type>,
                    }
                })
                .collect::<Vec<_>>()
        } else {
            panic!("Only structs with named fields are supported");
        }
    } else {
        panic!("Only structs are supported");
    };

    let derives = quote! { Default, #(#derived_traits),* };

    #[cfg(feature = "rkyv")]
    let derives = quote! { #derives, ::rkyv::Archive, ::rkyv::Deserialize, ::rkyv::Serialize };

    #[cfg(feature = "serde")]
    let derives = quote! { #derives, ::serde::Deserialize, ::serde::Serialize };

    // Generate the new struct
    let expanded = quote! {
        #[derive(#derives)]
        pub struct #opt_name {
            #(#fields)*
        }
    };

    // Convert into TokenStream
    TokenStream::from(expanded)
}
