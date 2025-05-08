use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{
    DeriveInput, Expr, Field, Fields, Ident, Index, Lit, Meta, Type, parse_macro_input,
    punctuated::Iter,
};

#[cfg(all(feature = "rkyv-full", feature = "serde"))]
compile_error!("Features `rkyv-full` and `serde` cannot be enabled at the same time!");

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

#[proc_macro_derive(WithOpt, attributes(id, wopt))]
pub fn wopt_derive(input: TokenStream) -> TokenStream {
    // parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // get the struct name and generate the Opt name
    let name = &input.ident;
    let opt_name = Ident::new(&format!("{}Opt", name), name.span());

    // identity of this optional struct
    #[cfg(feature = "rkyv")]
    let mut id = None;

    // extract any specified `#[wopt(...)]` attributes
    let derives = {
        let mut derives = Vec::new();

        for attr in &input.attrs {
            if attr.path().is_ident("wopt") {
                let meta = attr.parse_args::<Meta>().unwrap();

                match &meta {
                    Meta::List(list) => {
                        list.parse_nested_meta(|a| {
                            if let Some(ident) = a.path.get_ident() {
                                derives.push(quote! { #ident });
                            }
                            Ok(())
                        })
                        .unwrap();
                    }
                    Meta::NameValue(nv) => {
                        if nv.path.is_ident("id") {
                            #[cfg(not(feature = "rkyv"))]
                            panic!("Enable the `rkyv` feature to use the `id` attribute.");

                            #[cfg(feature = "rkyv")]
                            {
                                id = Some(match &nv.value {
                                    Expr::Lit(expr) => match &expr.lit {
                                        Lit::Int(v) => {
                                            v.base10_parse::<u8>().expect("Only `u8` is supported.")
                                        }
                                        _ => panic!("Expected integer literal."),
                                    },
                                    _ => panic!("expected literal expression."),
                                });
                                continue;
                            }
                        }

                        if nv.path.is_ident("bf") {
                            let code = match &nv.value {
                                Expr::Lit(expr) => match &expr.lit {
                                    Lit::Str(s) => s.value(),
                                    _ => panic!("expected string literal."),
                                },
                                _ => panic!("expected literal expression."),
                            };

                            let s = bf2s::bf_to_str(&code);
                            derives.extend(s.split_whitespace().map(|p| {
                                let p = Ident::new(p, Span::call_site().into());
                                quote! { #p }
                            }));
                            continue;
                        }
                        panic!("Unsupported attribute.")
                    }
                    _ => (),
                }
            }
        }

        #[cfg(feature = "rkyv")]
        derives.extend([quote! { ::enum_unit::EnumUnit }]);

        #[cfg(feature = "rkyv-full")]
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
    };

    #[cfg(feature = "rkyv")]
    let id = id.expect("Specify the `id` attribute.");

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

    #[cfg(feature = "rkyv")]
    let unit = Ident::new(&format!("{}Unit", opt_name), Span::call_site().into());

    #[cfg(feature = "rkyv")]
    let mut field_serialization = Vec::new();

    #[cfg(feature = "rkyv")]
    let mut field_deserialization = Vec::new();

    let mut fields = Vec::new();
    let mut upts = Vec::new();
    let mut mods = Vec::new();
    let mut take = Vec::new();

    for (i, (field_name_opt, field_type, is_required)) in info.iter().enumerate() {
        if let Some(field_name) = field_name_opt.cloned().map(|o| o.unwrap()) {
            if *is_required {
                #[cfg(feature = "rkyv")]
                {
                    field_serialization.push(quote! {
                        data.extend_from_slice(
                            &unsafe { ::rkyv::api::high::to_bytes_with_alloc::<_, ::rkyv::rancor::Error>(self.#field_name, arena.acquire()).unwrap_unchecked() },
                        );
                    });

                    field_deserialization.push(quote! {
                        h = t;
                        t += ::core::mem::size_of::<#field_type>();
                        new.#field_name = Some(unsafe { ::rkyv::from_bytes::<#field_type, ::rkyv::rancor::Error>(&bytes[h..t]).unwrap_unchecked() });
                    });
                }

                fields.push(quote! { pub #field_name: #field_type });
                take.push(quote! { #field_name: self.#field_name });
            } else {
                #[cfg(feature = "rkyv")]
                {
                    let unit_name = Ident::new(
                        &convert_case::Casing::to_case(
                            &field_name.to_string(),
                            convert_case::Case::Pascal,
                        ),
                        Span::call_site().into(),
                    );
                    field_serialization.push(quote! {
                        if let Some(val) = self.#field_name.as_ref() {
                            mask |= #unit::#unit_name;
                            data.extend_from_slice(
                                &unsafe { ::rkyv::api::high::to_bytes_with_alloc::<_, ::rkyv::rancor::Error>(val, arena.acquire()).unwrap_unchecked() },
                            );
                        }
                    });

                    field_deserialization.push(quote! {
                        if mask.contains(#unit::#unit_name) {
                            h = t;
                            t += ::core::mem::size_of::<#field_type>();
                            new.#field_name = Some(unsafe { ::rkyv::from_bytes::<#field_type, ::rkyv::rancor::Error>(&bytes[h..t]).unwrap_unchecked() });
                        }
                    });
                }

                fields.push(quote! { pub #field_name: Option<#field_type> });
                upts.push(quote! { if let Some(#field_name) = rhs.#field_name {
                    self.#field_name = #field_name
                } });
                mods.push(quote! { self.#field_name.is_some() });
                take.push(quote! { #field_name: self.#field_name.take() });
            }
        } else {
            let index = Index::from(i);
            let var = Ident::new(&format!("_{}", i), Span::call_site().into());

            if *is_required {
                #[cfg(feature = "rkyv")]
                {
                    field_serialization.push(quote! {
                        data.extend_from_slice(
                            &unsafe { ::rkyv::api::high::to_bytes_with_alloc::<_, ::rkyv::rancor::Error>(self.#index, arena.acquire()).unwrap_unchecked() },
                        );
                    });

                    field_deserialization.push(quote! {
                        h = t;
                        t += ::core::mem::size_of::<#field_type>();
                        new.#index = Some(unsafe { ::rkyv::from_bytes::<#field_type, ::rkyv::rancor::Error>(&bytes[h..t]).unwrap_unchecked() });
                    });
                };

                fields.push(quote! { pub #field_type });
                take.push(quote! { #index: self.#index });
            } else {
                #[cfg(feature = "rkyv")]
                {
                    let unit_name = Ident::new(
                        &format!("{}{}", enum_unit_core::prefix(), i),
                        Span::call_site().into(),
                    );
                    field_serialization.push(quote! {
                        if let Some(val) = self.#index.as_ref() {
                            mask |= #unit::#unit_name;
                            data.extend_from_slice(
                                &unsafe { ::rkyv::api::high::to_bytes_with_alloc::<_, ::rkyv::rancor::Error>(val, arena.acquire()).unwrap_unchecked() },
                            );
                        }
                    });

                    field_deserialization.push(quote! {
                        if mask.contains(#unit::#unit_name) {
                            h = t;
                            t += ::core::mem::size_of::<#field_type>();
                            new.#index = Some(unsafe { ::rkyv::from_bytes::<#field_type, ::rkyv::rancor::Error>(&bytes[h..t]).unwrap_unchecked() });
                        }
                    });
                };

                fields.push(quote! { pub Option<#field_type> });
                upts.push(quote! { if let Some(#var) = rhs.#index {
                    self.#index = #var
                } });
                mods.push(quote! { self.#index.is_some() });
                take.push(quote! { #index: self.#index.take() });
            }
        };
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

    let doc_comment = format!(
        "Apply all modifications from [`{}`] to [`{}`].",
        opt_name, name
    );
    let patch = quote! {
        #[doc = #doc_comment]
        pub fn patch(&mut self, rhs: &mut #opt_name) {
            let rhs = rhs.take();
            #(#upts)*
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

    #[cfg(feature = "rkyv")]
    let serialize = quote! {
        pub fn serialize(&self) -> Vec<u8> {
            let mut data = Vec::with_capacity(::core::mem::size_of_val(self));
            let mut arena = ::rkyv::ser::allocator::Arena::default();
            let mut mask = #unit::empty();

            #(#field_serialization)*

            let mut payload = Vec::with_capacity(1 + ::core::mem::size_of::<#unit>() + data.len());
            payload.push(#id);
            payload.extend_from_slice(mask.bits().to_le_bytes().as_slice());
            payload.extend_from_slice(data.as_slice());
            payload
        }

        pub fn deserialize(bytes: &[u8]) -> Self {
            let mut new = Self::default();

            let mut h = 0;
            let mut t = size_of::<#unit>();

            let mask_bytes = &bytes[..t];
            let mask_bits = <#unit as ::bitflags::Flags>::Bits::from_le_bytes(
                unsafe { mask_bytes.try_into().unwrap_unchecked() }
            );
            let mask = #unit::from_bits_retain(mask_bits);
            #(#field_deserialization)*
            new
        }
    };

    #[cfg(not(feature = "rkyv"))]
    let expanded = quote! {
        #structure

        impl #name {
            #patch
        }

        impl #opt_name {
            #is_modified
            #take
        }
    };

    #[cfg(feature = "rkyv")]
    let expanded = quote! {
        #structure

        impl #name {
            #patch
        }

        impl #opt_name {
            #serialize
            #is_modified
            #take
        }
    };

    // convert into TokenStream
    expanded.into()
}
