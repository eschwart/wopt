use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{
    DeriveInput, Expr, Field, Fields, Ident, Index, Lit, LitStr, Meta, Type, parse_macro_input,
    punctuated::Iter,
};

#[cfg(all(not(feature = "rkyv"), feature = "unchecked"))]
compile_error!("Feature `unchecked` requires feature `rkyv`.");

fn get_field_kvs(
    fields: Iter<Field>,
    is_named: bool,
) -> Vec<(
    Option<&Option<Ident>>,
    &Type,
    bool,
    bool,
    Option<[Ident; 2]>,
)> {
    fields
        .map(|field: &Field| {
            if field.attrs.len() > 1 {
                panic!("Only 1 attribute per field is supported.")
            }
            let (mut is_required, mut is_skipped) = Default::default();
            let (mut ser, mut de) = Default::default();

            if let Some(attr) = field.attrs.first() {
                if attr.path().is_ident("wopt") {
                    attr.parse_nested_meta(|a| {
                        if let Some(ident) = a.path.get_ident() {
                            match ident.to_string().as_str() {
                                "required" => is_required = true,
                                "skip" => is_skipped = true,
                                "ser" => {
                                    let value = a.value()?;
                                    let s: LitStr = value.parse()?;
                                    ser = Some(s.value())
                                }
                                "de" => {
                                    let value = a.value()?;
                                    let s: LitStr = value.parse()?;
                                    de = Some(s.value())
                                }
                                attr => panic!("Unsupported attribute ({}).", attr),
                            }
                        }
                        Ok(())
                    })
                    .unwrap();

                    if is_required && is_skipped {
                        panic!("`required` and `skip` can't be specified together.")
                    }
                }
            }

            // check if ser/de is complete, is provided
            let mut _serde_fn = None;
            match (ser, de) {
                (None, None) => (),
                (Some(ser), Some(de)) => {
                    _serde_fn = Some([
                        Ident::new(&ser, Span::call_site().into()),
                        Ident::new(&de, Span::call_site().into()),
                    ])
                }
                _ => panic!("Both ser/de need to be implemented."),
            }

            if is_named {
                (
                    Some(&field.ident),
                    &field.ty,
                    is_required,
                    is_skipped,
                    _serde_fn,
                )
            } else {
                (None, &field.ty, is_required, is_skipped, _serde_fn)
            }
        })
        .collect()
}

#[proc_macro_derive(WithOpt, attributes(id, wopt))]
pub fn wopt_derive(input: TokenStream) -> TokenStream {
    // parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // get the struct name
    let name = &input.ident;

    // identity of this optional struct
    #[cfg(feature = "rkyv")]
    let mut id = None;

    #[allow(unused_mut)]
    let mut is_unit = false;

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
            _ => {
                #[cfg(not(feature = "rkyv"))]
                panic!("Unit structs are only supported with the `rkyv` feature.");

                #[cfg(feature = "rkyv")]
                {
                    is_unit = true;
                    vec![]
                }
            }
        }
    } else {
        panic!("Only structs are supported");
    };

    // process any `#[wopt(...)]` attributes
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
                                            let value = v
                                                .base10_parse::<u8>()
                                                .expect("Only `u8` is supported.");
                                            if value > 127 {
                                                panic!("Value too large (max: 127)")
                                            }
                                            value
                                        }
                                        _ => panic!("Expected integer literal."),
                                    },
                                    _ => panic!("Expected literal expression."),
                                });
                                continue;
                            }
                        }
                        if nv.path.is_ident("bf") {
                            #[cfg(not(feature = "bf"))]
                            panic!("Enable the `bf` feature to use brainfuck.");

                            #[cfg(feature = "bf")]
                            {
                                let code = match &nv.value {
                                    Expr::Lit(expr) => match &expr.lit {
                                        Lit::Str(s) => s.value(),
                                        _ => panic!("Expected string literal."),
                                    },
                                    _ => panic!("Expected literal expression."),
                                };

                                let s = bf2s::bf_to_str(&code);
                                derives.extend(s.split_whitespace().map(|p| {
                                    let p = Ident::new(p, Span::call_site().into());
                                    quote! { #p }
                                }));
                                continue;
                            }
                        }
                        panic!("Unsupported attribute.")
                    }
                    _ => (),
                }
            }
        }
        #[cfg(feature = "rkyv")]
        if !is_unit {
            derives.extend([quote! { ::enum_unit::EnumUnit }]);
        }
        derives
    };

    #[cfg(feature = "rkyv")]
    let id_og = id.expect("Specify the `id` attribute.");
    #[cfg(feature = "rkyv")]
    let id_opt = id_og + i8::MAX as u8;

    let opt_name = if is_unit {
        name.clone()
    } else {
        Ident::new(&format!("{}Opt", name), name.span())
    };

    #[cfg(feature = "rkyv")]
    let unit = Ident::new(&format!("{}Unit", opt_name), Span::call_site().into());

    #[cfg(feature = "rkyv")]
    let mut field_serialization = Vec::new();

    #[cfg(feature = "rkyv")]
    let mut field_deserialization = Vec::new();

    #[cfg(feature = "rkyv")]
    let mut field_deserialization_new = Vec::new();

    #[cfg(feature = "rkyv")]
    let mut field_serialization_opt = Vec::new();

    #[cfg(feature = "rkyv")]
    let mut field_deserialization_opt = Vec::new();

    let mut fields = Vec::new();
    let mut upts = Vec::new();
    let mut mods = Vec::new();
    let mut take = Vec::new();

    #[cfg(all(feature = "rkyv", not(feature = "unchecked")))]
    let unwrap = Ident::new("unwrap", Span::call_site().into());

    #[cfg(all(feature = "rkyv", feature = "unchecked"))]
    let unwrap = Ident::new("unwrap_unchecked", Span::call_site().into());

    for (i, (field_name_opt, field_type, is_required, is_skipped, _serde_fn)) in
        info.iter().enumerate()
    {
        if let Some(field_name) = field_name_opt.cloned().map(|o| o.unwrap()) {
            #[cfg(feature = "rkyv")]
            {
                if let Some([ser, de]) = _serde_fn {
                    field_serialization.push(quote! {
                        data.extend_from_slice(
                            #ser(&self.#field_name).as_ref(),
                        );
                    });
                    field_deserialization.push(quote! {
                        h = t;
                        t += ::core::mem::size_of::<#field_type>();
                        let #field_name = #de(&bytes[h..t]);
                    });
                } else {
                    field_serialization.push(quote! {
                        data.extend_from_slice(
                            &unsafe { ::rkyv::api::high::to_bytes_with_alloc::<_, ::rkyv::rancor::Error>(&self.#field_name, arena.acquire()).#unwrap() },
                        );
                    });
                    field_deserialization.push(quote! {
                        h = t;
                        t += ::core::mem::size_of::<#field_type>();
                        let #field_name = unsafe { ::rkyv::from_bytes::<#field_type, ::rkyv::rancor::Error>(&bytes[h..t]).#unwrap() };
                    });
                }
                field_deserialization_new.push(quote! {
                    #field_name
                });
            }

            if *is_skipped {
                continue;
            }

            if *is_required {
                #[cfg(feature = "rkyv")]
                {
                    if let Some([ser, de]) = _serde_fn {
                        field_serialization_opt.push(quote! {
                            data.extend_from_slice(
                                #ser(&self.#field_name).as_ref(),
                            );
                        });
                        field_deserialization_opt.push(quote! {
                            h = t;
                            t += ::core::mem::size_of::<#field_type>();
                            new.#field_name = #de(&bytes[h..t]);
                        });
                    } else {
                        field_serialization_opt.push(quote! {
                            data.extend_from_slice(
                                &unsafe { ::rkyv::api::high::to_bytes_with_alloc::<_, ::rkyv::rancor::Error>(&self.#field_name, arena.acquire()).#unwrap() },
                            );
                        });
                        field_deserialization_opt.push(quote! {
                            h = t;
                            t += ::core::mem::size_of::<#field_type>();
                            new.#field_name = unsafe { ::rkyv::from_bytes::<#field_type, ::rkyv::rancor::Error>(&bytes[h..t]).#unwrap() };
                        });
                    }
                }
                fields.push(quote! { pub #field_name: #field_type });
                take.push(quote! { #field_name: self.#field_name });
            } else {
                #[cfg(feature = "rkyv")]
                if !is_unit {
                    let unit_name = Ident::new(
                        &convert_case::Casing::to_case(
                            &field_name.to_string(),
                            convert_case::Case::Pascal,
                        ),
                        Span::call_site().into(),
                    );

                    if let Some([ser, de]) = _serde_fn {
                        field_serialization_opt.push(quote! {
                            if let Some(val) = self.#field_name.as_ref() {
                                mask |= #unit::#unit_name;
                                data.extend_from_slice(
                                    #ser(val).as_ref(),
                                );
                            }
                        });
                        field_deserialization_opt.push(quote! {
                            h = t;
                            t += ::core::mem::size_of::<#field_type>();
                            new.#field_name = Some(#de(&bytes[h..t]));
                        });
                    } else {
                        field_serialization_opt.push(quote! {
                            if let Some(val) = self.#field_name.as_ref() {
                                mask |= #unit::#unit_name;
                                data.extend_from_slice(
                                    &unsafe { ::rkyv::api::high::to_bytes_with_alloc::<_, ::rkyv::rancor::Error>(val, arena.acquire()).#unwrap() },
                                );
                            }
                        });

                        field_deserialization_opt.push(quote! {
                            if mask.contains(#unit::#unit_name) {
                                h = t;
                                t += ::core::mem::size_of::<#field_type>();
                                new.#field_name = Some(unsafe { ::rkyv::from_bytes::<#field_type, ::rkyv::rancor::Error>(&bytes[h..t]).#unwrap() });
                            }
                        });
                    }
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

            #[cfg(feature = "rkyv")]
            {
                if let Some([ser, de]) = _serde_fn {
                    field_serialization.push(quote! {
                        data.extend_from_slice(
                            #ser(&self.#index).as_ref(),
                        );
                    });
                    field_deserialization.push(quote! {
                        h = t;
                        t += ::core::mem::size_of::<#field_type>();
                        let #var = #de(&bytes[h..t]);
                    });
                } else {
                    field_serialization.push(quote! {
                        data.extend_from_slice(
                            &unsafe { ::rkyv::api::high::to_bytes_with_alloc::<_, ::rkyv::rancor::Error>(&self.#index, arena.acquire()).#unwrap() },
                        );
                    });
                    field_deserialization.push(quote! {
                        h = t;
                        t += ::core::mem::size_of::<#field_type>();
                        let #var = unsafe { ::rkyv::from_bytes::<#field_type, ::rkyv::rancor::Error>(&bytes[h..t]).#unwrap() };
                    });
                }
                field_deserialization_new.push(quote! {
                    #index: #var
                });
            }

            if *is_skipped {
                continue;
            }

            if *is_required {
                #[cfg(feature = "rkyv")]
                if let Some([ser, de]) = _serde_fn {
                    field_serialization_opt.push(quote! {
                        data.extend_from_slice(
                            #ser(&self.#index).as_ref(),
                        );
                    });
                    field_deserialization_opt.push(quote! {
                        h = t;
                        t += ::core::mem::size_of::<#field_type>();
                        new.#index = #de(&bytes[h..t]);
                    });
                } else {
                    field_serialization_opt.push(quote! {
                        data.extend_from_slice(
                            &unsafe { ::rkyv::api::high::to_bytes_with_alloc::<_, ::rkyv::rancor::Error>(&self.#index, arena.acquire()).#unwrap() },
                        );
                    });

                    field_deserialization_opt.push(quote! {
                        h = t;
                        t += ::core::mem::size_of::<#field_type>();
                        new.#index = unsafe { ::rkyv::from_bytes::<#field_type, ::rkyv::rancor::Error>(&bytes[h..t]).#unwrap() };
                    });
                }
                fields.push(quote! { pub #field_type });
                take.push(quote! { #index: self.#index });
            } else {
                #[cfg(feature = "rkyv")]
                if !is_unit {
                    let unit_name = Ident::new(
                        &format!("{}{}", enum_unit_core::prefix(), i),
                        Span::call_site().into(),
                    );

                    if let Some([ser, de]) = _serde_fn {
                        field_serialization_opt.push(quote! {
                            if let Some(val) = self.#index.as_ref() {
                                mask |= #unit::#unit_name;
                                data.extend_from_slice(
                                    #ser(val).as_ref(),
                                );
                            }
                        });
                        field_deserialization_opt.push(quote! {
                            h = t;
                            t += ::core::mem::size_of::<#field_type>();
                            new.#index = Some(#de(&bytes[h..t]));
                        });
                    } else {
                        field_serialization_opt.push(quote! {
                            if let Some(val) = self.#index.as_ref() {
                                mask |= #unit::#unit_name;
                                data.extend_from_slice(
                                    &unsafe { ::rkyv::api::high::to_bytes_with_alloc::<_, ::rkyv::rancor::Error>(val, arena.acquire()).#unwrap() },
                                );
                            }
                        });

                        field_deserialization_opt.push(quote! {
                            if mask.contains(#unit::#unit_name) {
                                h = t;
                                t += ::core::mem::size_of::<#field_type>();
                                new.#index = Some(unsafe { ::rkyv::from_bytes::<#field_type, ::rkyv::rancor::Error>(&bytes[h..t]).#unwrap() });
                            }
                        });
                    }
                }
                fields.push(quote! { pub Option<#field_type> });
                upts.push(quote! { if let Some(#var) = rhs.#index {
                    self.#index = #var
                } });
                mods.push(quote! { self.#index.is_some() });
                take.push(quote! { #index: self.#index.take() });
            }
        };
    }

    #[cfg(feature = "rkyv")]
    let (serde_og, serde_opt) = if is_unit {
        let serde = quote! {
            pub const fn serialize() -> [u8; 1] {
                [#id_og]
            }
        };
        (serde, quote! {})
    } else {
        let serde_og = quote! {
            pub fn serialize(&self) -> Vec<u8> {
                let mut data = Vec::with_capacity(::core::mem::size_of_val(self));
                let mut arena = ::rkyv::ser::allocator::Arena::default();

                #(#field_serialization)*

                let mut payload = Vec::with_capacity(1 + data.len());
                payload.push(#id_og);
                payload.extend_from_slice(data.as_slice());
                payload
            }

            pub fn deserialize(bytes: &[u8]) -> Self {
                let mut h = 0;
                let mut t = 0;

                #(#field_deserialization)*

                Self { #(#field_deserialization_new),* }
            }
        };

        let serde_opt = quote! {
            pub fn serialize(&self) -> Vec<u8> {
                let mut data = Vec::with_capacity(::core::mem::size_of_val(self));
                let mut arena = ::rkyv::ser::allocator::Arena::default();
                let mut mask = #unit::empty();

                #(#field_serialization_opt)*

                let mut payload = Vec::with_capacity(1 + ::core::mem::size_of::<#unit>() + data.len());
                payload.push(#id_opt);
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
                    unsafe { mask_bytes.try_into().#unwrap() }
                );
                let mask = #unit::from_bits_retain(mask_bits);
                #(#field_deserialization_opt)*
                new
            }
        };
        (serde_og, serde_opt)
    };

    // this is just filthy
    if is_unit {
        #[cfg(not(feature = "rkyv"))]
        return quote! {}.into();

        #[cfg(feature = "rkyv")]
        return quote! {
            impl #name {
                pub const ID: u8 = #id_og;
                #serde_og
            }
        }
        .into();
    }

    // generate the new struct
    let structure = if is_named {
        quote! {
            #[derive(#(#derives),*)]
            pub struct #opt_name {
                #(#fields),*
            }
        }
    } else if is_unit {
        quote! {}
    } else {
        quote! {
            #[derive(#(#derives),*)]
            pub struct #opt_name(#(#fields),*);
        }
    };

    let (impl_name, impl_opt_name) = if upts.is_empty() || is_unit {
        Default::default()
    } else {
        let patch = quote! {
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

        (
            quote! {
                #patch
            },
            quote! {
                #is_modified
                #take
            },
        )
    };

    #[cfg(feature = "rkyv")]
    let impl_name_id = quote! {
        pub const ID: u8 = #id_og;
    };
    #[cfg(not(feature = "rkyv"))]
    let impl_name_id = quote! {};

    #[cfg(feature = "rkyv")]
    let impl_name = quote! {
        #impl_name
        #serde_og
    };
    let impl_name = quote! {
        impl #name {
            #impl_name_id
            #impl_name
        }
    };

    #[cfg(feature = "rkyv")]
    let impl_opt_id = quote! {
        pub const ID: u8 = #id_opt;
    };
    #[cfg(not(feature = "rkyv"))]
    let impl_opt_id = quote! {};

    #[cfg(feature = "rkyv")]
    let impl_opt_name = quote! {
        #impl_opt_name
        #serde_opt
    };
    let impl_opt_name = quote! {
        impl #opt_name {
            #impl_opt_id
            #impl_opt_name
        }
    };

    quote! {
        #structure
        #impl_name
        #impl_opt_name
    }
    .into()
}
