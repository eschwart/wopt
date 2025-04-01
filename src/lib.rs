use proc_macro::TokenStream;
use quote::quote;
use syn::{
    DeriveInput, Field, Fields, Ident, Index, Meta, Type, parse_macro_input, punctuated::Iter,
};

#[cfg(all(feature = "rkyv", feature = "serde"))]
compile_error!("Features `rkyv` and `serde` cannot be enabled at the same time!");

// function to extract `#[wopt(derive(...))]`
fn get_derivations(input: &DeriveInput) -> Vec<proc_macro2::TokenStream> {
    let mut derives = Vec::new();

    for attr in &input.attrs {
        if attr.path().is_ident("wopt") {
            let meta = attr.parse_args::<Meta>().unwrap();
            let meta_list = meta.require_list().unwrap();

            meta_list
                .parse_nested_meta(|a| {
                    if let Some(ident) = a.path.get_ident() {
                        derives.push(quote! { #ident });
                    }
                    Ok(())
                })
                .unwrap();
        }
    }

    #[cfg(feature = "rkyv")]
    derives.extend([
        quote! { ::rkyv::Archive },
        quote! { ::rkyv::Serialize },
        quote! { ::rkyv::Deserialize },
    ]);

    #[cfg(feature = "serde")]
    derives.extend([
        quote! { ::serde::Serialize },
        quote! { ::serde::Deserialize },
    ]);

    derives
}

fn get_field_kvs(
    fields: Iter<Field>,
    is_named: bool,
) -> Vec<(Option<&Option<Ident>>, &Type, bool)> {
    fields
        .filter_map(|field: &Field| {
            if field.attrs.len() > 1 {
                panic!("Only 1 attribute per field is supported.")
            }
            let (mut is_required, mut skip) = Default::default();

            if let Some(attr) = field.attrs.first() {
                if attr.path().is_ident("wopt") {
                    let mut n = 0;
                    attr.parse_nested_meta(|a| {
                        if let Some(ident) = a.path.get_ident() {
                            match ident.to_string().as_str() {
                                "required" => is_required = true,
                                "skip" => skip = true,
                                _ => panic!(
                                    "Only `required` & `skip` field attributes are supported."
                                ),
                            }
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

            if skip {
                return None;
            }

            if is_named {
                Some((Some(&field.ident), &field.ty, is_required))
            } else {
                Some((None, &field.ty, is_required))
            }
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
    let derives = get_derivations(&input);

    // the type of struct
    let mut is_named = false;

    // match on the fields of the struct
    let info: Vec<_> = if let syn::Data::Struct(ref data) = input.data {
        match &data.fields {
            Fields::Named(fields) => {
                is_named = true;
                get_field_kvs(fields.named.iter(), true)
            }
            Fields::Unnamed(fields) => get_field_kvs(fields.unnamed.iter(), false),
            _ => panic!("Unit structs are not supported."),
        }
    } else {
        panic!("Only structs are supported");
    };

    let mut fields = Vec::new();
    let mut mods = Vec::new();
    let mut take = Vec::new();

    for (i, (field_name_opt, field_type, is_required)) in info.into_iter().enumerate() {
        if let Some(field_name) = field_name_opt {
            if is_required {
                fields.push(quote! { pub #field_name: #field_type });
                take.push(quote! { #field_name: self.#field_name });
            } else {
                fields.push(quote! { pub #field_name: Option<#field_type> });
                mods.push(quote! { self.#field_name.is_some() });
                take.push(quote! { #field_name: self.#field_name.take() });
            }
        } else {
            let index = Index::from(i);

            if is_required {
                fields.push(quote! { pub #field_type });
                take.push(quote! { #index: self.#index });
            } else {
                fields.push(quote! { pub Option<#field_type> });
                mods.push(quote! { self.#index.is_some() });
                take.push(quote! { #index: self.#index.take() });
            }
        }
    }

    // generate the new struct
    let structure = if is_named {
        quote! {
            #[derive(#(#derives),*)]
            pub struct #opt_name {
                #(#fields),*
            }
        }
    } else {
        quote! {
            #[derive(#(#derives),*)]
            pub struct #opt_name(#(#fields),*);
        }
    };

    let is_modified = quote! {
        pub const fn is_modified(&self) -> bool {
            #(#mods)||*
        }
    };

    let take = quote! {
        pub const fn take(&mut self) -> Self {
            Self {
                #(#take),*
            }
        }
    };

    let expanded = quote! {
        #structure

        impl #opt_name {
            #is_modified
            #take
        }
    };

    // convert into TokenStream
    expanded.into()
}
