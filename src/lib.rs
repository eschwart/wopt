use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{
    DeriveInput, Field, Fields, Ident, Index, LitStr, Meta, Path, PathSegment, Type, TypePath,
    parse_macro_input, punctuated::Iter,
};

#[cfg(any(feature = "bf", feature = "bytemuck"))]
use syn::{Expr, Lit};

#[cfg(all(not(feature = "bytemuck"), feature = "unchecked"))]
compile_error!("Feature `unchecked` requires feature `bytemuck`.");

fn get_opt_type(original: &Type) -> Type {
    if let Type::Path(TypePath { path, .. }) = original {
        if let Some(last_segment) = path.segments.last() {
            let orig_ident = &last_segment.ident;
            let new_ident = Ident::new(
                format!("{orig_ident}Opt").as_str(),
                Span::call_site().into(),
            );

            // Construct a new path with the modified ident and same arguments
            let new_segment = PathSegment {
                ident: new_ident,
                arguments: last_segment.arguments.clone(),
            };

            let mut new_path = path.clone();
            new_path.segments.pop();
            new_path.segments.push(new_segment);

            return Type::Path(TypePath {
                qself: None,
                path: new_path,
            });
        }
    }
    panic!("Unexpected syn::Type variant.")
}

struct FieldAttrs<'a> {
    field_name_opt: Option<&'a Option<Ident>>,
    field_type: &'a Type,
    field_type_opt: Type,
    is_optional: bool,
    is_required: bool,
    is_skipped: bool,
    _is_serde: bool,
    _serde_fn: Option<[Path; 2]>,
}

fn get_field_kvs(fields: Iter<Field>, is_named: bool) -> Vec<FieldAttrs> {
    fields
        .map(|field: &Field| {
            let (mut is_optional, mut is_required, mut is_skipped, mut _is_serde) =
                Default::default();
            let (mut ser, mut de) = Default::default();

            if let Some(attr) = field.attrs.first() {
                if attr.path().is_ident("wopt") {
                    attr.parse_nested_meta(|a| {
                        if let Some(ident) = a.path.get_ident() {
                            match ident.to_string().as_str() {
                                "optional" => is_optional = true,
                                "required" => is_required = true,
                                "skip" => is_skipped = true,
                                "serde" => _is_serde = true,
                                "ser" => {
                                    let value = a.value()?;
                                    let s: LitStr = value.parse()?;
                                    let p = syn::parse_str::<Path>(s.value().as_str())?;
                                    ser = Some(p)
                                }
                                "de" => {
                                    let value = a.value()?;
                                    let s: LitStr = value.parse()?;
                                    let p = syn::parse_str::<Path>(s.value().as_str())?;
                                    de = Some(p)
                                }
                                attr => panic!("Unsupported attribute ({attr})."),
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

            // determine if optional struct provided
            let field_type = &field.ty;
            let field_type_opt = if is_optional {
                get_opt_type(field_type)
            } else {
                field_type.clone()
            };

            // override if any user-provided definitions
            let _serde_fn = match (ser, de) {
                (None, None) => None,
                (Some(ser), Some(de)) => Some([ser, de]),
                _ => panic!("Both ser/de need to be implemented."),
            };

            FieldAttrs {
                field_name_opt: is_named.then_some(&field.ident),
                field_type,
                field_type_opt,
                is_optional,
                is_required,
                is_skipped,
                _is_serde,
                _serde_fn,
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
    #[cfg(feature = "bytemuck")]
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
                #[cfg(not(feature = "bytemuck"))]
                panic!("Unit structs are only supported with the `bytemuck` feature.");

                #[cfg(feature = "bytemuck")]
                {
                    is_unit = true;
                    vec![]
                }
            }
        }
    } else {
        panic!("Only structs are supported");
    };

    if info.is_empty() && !is_unit {
        panic!("Must have at least 1 field.")
    }

    let mut derives = Vec::new();
    let mut _no_serde = false;

    // process any `#[wopt(...)]` attributes
    for attr in &input.attrs {
        if attr.path().is_ident("wopt") {
            let meta = attr.parse_args::<Meta>().unwrap();

            match &meta {
                Meta::Path(path) => {
                    if !path.is_ident("no_serde") {
                        panic!("Only 'no_serde' path meta is supported.")
                    }
                    _no_serde = true
                }

                Meta::List(list) => {
                    if !list.path.is_ident("derive") {
                        panic!("Only 'derive' list meta is supported.")
                    }

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
                        #[cfg(not(feature = "bytemuck"))]
                        panic!("Enable the `bytemuck` feature to use the `id` attribute.");

                        #[cfg(feature = "bytemuck")]
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
            }
        }
    }
    #[cfg(feature = "bytemuck")]
    if !is_unit {
        derives.extend([quote! { ::enum_unit::EnumUnit }]);
    }

    #[cfg(feature = "bytemuck")]
    let id_og = id.expect("Specify the `id` attribute.");
    #[cfg(feature = "bytemuck")]
    let id_opt = id_og + (i8::MAX as u8);

    let opt_name = if is_unit {
        name.clone()
    } else {
        Ident::new(&format!("{name}Opt"), name.span())
    };

    #[cfg(feature = "bytemuck")]
    let unit = Ident::new(&format!("{opt_name}Unit"), Span::call_site().into());

    let mut field_struct_new = Vec::new();

    #[cfg(feature = "bytemuck")]
    let mut field_serialization = Vec::new();

    #[cfg(feature = "bytemuck")]
    let mut field_deserialization = Vec::new();

    #[cfg(feature = "bytemuck")]
    let mut field_serialization_opt = Vec::new();

    #[cfg(feature = "bytemuck")]
    let mut field_deserialization_opt = Vec::new();

    let mut fields = Vec::new();
    let mut upts = Vec::new();
    let mut mods = Vec::new();
    let mut take = Vec::new();
    let mut into = Vec::new();

    let mut size = Vec::new();
    let mut size_opt = Vec::new();

    #[cfg(all(feature = "bytemuck", not(feature = "unchecked")))]
    let unwrap = Ident::new("unwrap", Span::call_site().into());

    #[cfg(all(feature = "bytemuck", feature = "unchecked"))]
    let unwrap = Ident::new("unwrap_unchecked", Span::call_site().into());

    let mut has_optional = false;

    for (
        i,
        FieldAttrs {
            field_name_opt,
            field_type,
            field_type_opt,
            is_optional,
            is_required,
            is_skipped,
            _is_serde,
            ref _serde_fn,
        },
    ) in info.into_iter().enumerate()
    {
        let (size_of, size_of_opt) = if _is_serde {
            (
                quote! { #field_type::UNPADDED_SIZE },
                quote! { 1 + #field_type_opt::UNPADDED_SIZE },
            )
        } else {
            (
                quote! { ::core::mem::size_of::<#field_type>() },
                quote! { ::core::mem::size_of::<#field_type_opt>() },
            )
        };

        if is_optional {
            has_optional = true
        }

        if let Some(field_name) = field_name_opt.cloned().map(|o| o.unwrap()) {
            field_struct_new.push(quote! { #field_name });

            #[cfg(feature = "bytemuck")]
            {
                if let Some([ser, de]) = _serde_fn {
                    field_serialization.push(quote! {
                        h = t; // PUT THIS AT THE END??
                        t += #size_of;
                        data[h..t].copy_from_slice(#ser(&self.#field_name).as_ref());
                    });
                    field_deserialization.push(quote! {
                        h = t;
                        t += #size_of;
                        let #field_name = #de(&bytes[h..t]);
                    });
                } else {
                    field_serialization.push(if _is_serde {
                        quote! {
                            h = t;
                            t += #size_of;
                            data[h..t].copy_from_slice(&self.#field_name.serialize()[1..]);
                        }
                    } else {
                        quote! {
                            h = t;
                            t += #size_of;
                            data[h..t].copy_from_slice(::bytemuck::bytes_of(&self.#field_name));
                        }
                    });
                    field_deserialization.push(if _is_serde {
                        quote! {
                            h = t;
                            t += #size_of;
                            let #field_name = #field_type::deserialize(&bytes[h..t]);
                        }
                    } else {
                        quote! {
                            h = t;
                            t += #size_of;
                            let #field_name = ::bytemuck::pod_read_unaligned(&bytes[h..t]);
                        }
                    });
                }
            }

            if is_skipped {
                continue;
            }

            if is_required {
                #[cfg(feature = "bytemuck")]
                {
                    if let Some([ser, de]) = _serde_fn {
                        field_serialization_opt.push(quote! {
                            data.extend_from_slice(#ser(&self.#field_name).as_ref());
                        });
                        field_deserialization_opt.push(quote! {
                            h = t;
                            t += #size_of_opt;
                            new.#field_name = #de(&bytes[h..t]);
                        });
                    } else {
                        field_serialization_opt.push(if _is_serde {
                            quote! {
                                data.extend_from_slice(&self.#field_name.serialize()[1..]);
                            }
                        } else {
                            quote! {
                                data.extend_from_slice(::bytemuck::bytes_of(&self.#field_name));
                            }
                        });
                        field_deserialization_opt.push(if _is_serde {
                            quote! {
                                h = t;
                                t += #size_of;
                                new.#field_name = #field_type_opt::deserialize(&bytes[h..t]);
                            }
                        } else {
                            quote! {
                                h = t;
                                t += #size_of;
                                new.#field_name = ::bytemuck::pod_read_unaligned(&bytes[h..t]);
                            }
                        });
                    }
                }
                fields.push(quote! { pub #field_name: #field_type_opt });
                take.push(quote! { #field_name: self.#field_name });
                into.push(quote! { #field_name: self.#field_name });
            } else {
                #[cfg(feature = "bytemuck")]
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
                                data.extend_from_slice(#ser(val).as_ref());
                            }
                        });
                        field_deserialization_opt.push(quote! {
                            h = t;
                            t += #size_of_opt;
                            new.#field_name = Some(#de(&bytes[h..t]));
                        });
                    } else {
                        field_serialization_opt.push(if is_optional {
                            quote! {
                                if self.#field_name.is_modified() {
                                    mask |= #unit::#unit_name;
                                    data.extend_from_slice(&self.#field_name.serialize()[1..]);
                                }
                            }
                        } else {
                            quote! {
                                if let Some(val) = self.#field_name.as_ref() {
                                    mask |= #unit::#unit_name;
                                    data.extend_from_slice(::bytemuck::bytes_of(val));
                                }
                            }
                        });

                        field_deserialization_opt.push(if is_optional {
                            quote! {
                                if mask.contains(#unit::#unit_name) {
                                    h = t;
                                    new.#field_name = #field_type_opt::deserialize_with(bytes, &mut h, &mut t);
                                }
                            }
                        } else {
                            quote! {
                                if mask.contains(#unit::#unit_name) {
                                    h = t;
                                    t += #size_of_opt;
                                    new.#field_name = Some(::bytemuck::pod_read_unaligned(&bytes[h..t]));
                                }
                            }
                        });
                    }
                }
                fields.push(if is_optional {
                    quote! { pub #field_name: #field_type_opt }
                } else {
                    quote! { pub #field_name: Option<#field_type_opt> }
                });
                upts.push(if is_optional {
                    quote! { if rhs.#field_name.is_modified() {
                        self.#field_name.patch(&mut rhs.#field_name)
                    } }
                } else {
                    quote! { if let Some(#field_name) = rhs.#field_name {
                        self.#field_name = #field_name
                    } }
                });
                mods.push(if is_optional {
                    quote! { self.#field_name.is_modified() }
                } else {
                    quote! { self.#field_name.is_some() }
                });
                take.push(quote! { #field_name: self.#field_name.take() });
                into.push(if is_optional {
                    quote! { #field_name: self.#field_name.into_opt() }
                } else {
                    quote! { #field_name: Some(self.#field_name) }
                });
            }
        } else {
            let index = Index::from(i);
            let var = Ident::new(&format!("_{i}"), Span::call_site().into());

            field_struct_new.push(quote! { #index: #var });

            #[cfg(feature = "bytemuck")]
            {
                if let Some([ser, de]) = _serde_fn {
                    field_serialization.push(quote! {
                        h = t;
                        t += #size_of;
                        data[h..t].copy_from_slice(#ser(&self.#index).as_ref());
                    });
                    field_deserialization.push(quote! {
                        h = t;
                        t += #size_of;
                        let #var = #de(&bytes[h..t]);
                    });
                } else {
                    field_serialization.push(if _is_serde {
                        quote! {
                            h = t;
                            t += #size_of;
                            data[h..t].copy_from_slice(&self.#index.serialize()[1..]);
                        }
                    } else {
                        quote! {
                            h = t;
                            t += #size_of;
                            data[h..t].copy_from_slice(::bytemuck::bytes_of(&self.#index));
                        }
                    });
                    field_deserialization.push(if _is_serde {
                        quote! {
                            h = t;
                            t += #size_of;
                            let #var = #field_type::deserialize(&bytes[h..t]);
                        }
                    } else {
                        quote! {
                            h = t;
                            t += #size_of;
                            let #var = ::bytemuck::pod_read_unaligned(&bytes[h..t]);
                        }
                    });
                }
            }

            if is_skipped {
                continue;
            }

            if is_required {
                #[cfg(feature = "bytemuck")]
                if let Some([ser, de]) = _serde_fn {
                    field_serialization_opt.push(quote! {
                        data.extend_from_slice(#ser(&self.#index).as_ref());
                    });
                    field_deserialization_opt.push(quote! {
                        h = t;
                        t += #size_of_opt;
                        new.#index = #de(&bytes[h..t]);
                    });
                } else {
                    field_serialization_opt.push(if _is_serde {
                        quote! {
                            data.extend_from_slice(&self.#index.serialize()[1..]);
                        }
                    } else {
                        quote! {
                            data.extend_from_slice(::bytemuck::bytes_of(&self.#index));
                        }
                    });
                    field_deserialization_opt.push(if _is_serde {
                        quote! {
                            h = t;
                            t += #size_of;
                            new.#index = #field_type_opt::deserialize(&bytes[h..t]);
                        }
                    } else {
                        quote! {
                            h = t;
                            t += #size_of;
                            new.#index = ::bytemuck::pod_read_unaligned(&bytes[h..t]);
                        }
                    });
                }
                fields.push(quote! { pub #field_type_opt });
                take.push(quote! { #index: self.#index });
                into.push(quote! { #index: self.#index });
            } else {
                #[cfg(feature = "bytemuck")]
                if !is_unit {
                    let unit_name = Ident::new(
                        &format!("{}{}", enum_unit_core::prefix(), i),
                        Span::call_site().into(),
                    );

                    if let Some([ser, de]) = _serde_fn {
                        field_serialization_opt.push(quote! {
                            if let Some(val) = self.#index.as_ref() {
                                mask |= #unit::#unit_name;
                                data.extend_from_slice(#ser(val).as_ref());
                            }
                        });
                        field_deserialization_opt.push(quote! {
                            h = t;
                            t += #size_of_opt;
                            new.#index = Some(#de(&bytes[h..t]));
                        });
                    } else {
                        field_serialization_opt.push(if is_optional {
                            quote! {
                                if self.#index.is_modified() {
                                    mask |= #unit::#unit_name;
                                    data.extend_from_slice(&self.#index.serialize()[1..]);
                                }
                            }
                        } else {
                            quote! {
                                if let Some(val) = self.#index.as_ref() {
                                    mask |= #unit::#unit_name;
                                    data.extend_from_slice(::bytemuck::bytes_of(val));
                                }
                            }
                        });

                        field_deserialization_opt.push(if is_optional {
                            quote! {
                                if mask.contains(#unit::#unit_name) {
                                    h = t;
                                    new.#index = #field_type_opt::deserialize_with(bytes, &mut h, &mut t)
                                }
                            }
                        } else {
                            quote! {
                                if mask.contains(#unit::#unit_name) {
                                    h = t;
                                    t += #size_of_opt;
                                    new.#index = Some(::bytemuck::pod_read_unaligned(&bytes[h..t]));
                                }
                            }
                        });
                    }
                }
                fields.push(if is_optional {
                    quote! { pub #field_type_opt }
                } else {
                    quote! { pub Option<#field_type_opt> }
                });

                upts.push(if is_optional {
                    quote! { if rhs.#index.is_modified() {
                        self.#index.patch(&mut rhs.#index)
                    } }
                } else {
                    quote! { if let Some(#var) = rhs.#index {
                        self.#index = #var
                    } }
                });
                mods.push(if is_optional {
                    quote! { self.#index.is_modified() }
                } else {
                    quote! { self.#index.is_some() }
                });
                take.push(quote! { #index: self.#index.take() });

                into.push(if is_optional {
                    quote! { #index: self.#index.into_opt() }
                } else {
                    quote! { #index: Some(self.#index) }
                });
            }
        };
        size.push(size_of);
        size_opt.push(size_of_opt);
    }

    #[cfg(feature = "bytemuck")]
    let (serde_og, serde_opt) = if is_unit {
        let serde = quote! {
            pub const fn serialize() -> [u8; 1] {
                [#id_og]
            }
        };
        (serde, quote! {})
    } else {
        let serde_og = if _no_serde {
            quote! {}
        } else {
            quote! {
                pub fn serialize(&self) -> [u8; 1 + Self::UNPADDED_SIZE] {
                    let mut data = [0; 1 + Self::UNPADDED_SIZE];
                    let [mut h, mut t] = [0, ::core::mem::size_of_val(&#id_og)];
                    data[0] = #id_og;
                    #(#field_serialization)*
                    data
                }

                pub fn deserialize(bytes: &[u8]) -> Self {
                    let [mut h, mut t] = [0; 2];
                    #(#field_deserialization)*
                    Self { #(#field_struct_new),* }
                }
            }
        };

        let try_into = quote! { mask_bytes.try_into().#unwrap() };
        #[cfg(feature = "unchecked")]
        let try_into = quote! {
            unsafe { #try_into }
        };

        let serde_opt = quote! {
            pub fn serialize(&self) -> Vec<u8> {
                let mut data = Vec::with_capacity(
                    1                               +   // identity byte
                    ::core::mem::size_of::<#unit>() +   // bitmask data
                    Self::UNPADDED_SIZE                 // field(s) data
                );
                data.push(#id_opt);
                let mut mask = #unit::empty();
                #(#field_serialization_opt)*
                data.splice(1..1, mask.bits().to_le_bytes());
                data
            }

            pub fn deserialize_with(bytes: &[u8], head: &mut usize, tail: &mut usize) -> Self {
                let mut h = *head;
                let mut t = h + ::core::mem::size_of::<#unit>();
                let mut new = Self::default();
                let mask_bytes = &bytes[h..t];
                let mask_bits = <#unit as ::bitflags::Flags>::Bits::from_le_bytes(#try_into);
                let mask = #unit::from_bits_retain(mask_bits);
                #(#field_deserialization_opt)*
                *head = h;
                *tail = t;
                new
            }

            pub fn deserialize(bytes: &[u8]) -> Self {
                let mut new = Self::default();
                let [mut h, mut t] = [0, ::core::mem::size_of::<#unit>()];
                let mask_bytes = &bytes[..t];
                let mask_bits = <#unit as ::bitflags::Flags>::Bits::from_le_bytes(#try_into);
                let mask = #unit::from_bits_retain(mask_bits);
                #(#field_deserialization_opt)*
                new
            }
        };
        (serde_og, serde_opt)
    };

    // this is just filthy
    if is_unit {
        #[cfg(not(feature = "bytemuck"))]
        return quote! {}.into();

        #[cfg(feature = "bytemuck")]
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

    let (impl_name, impl_name_opt) = if upts.is_empty() || is_unit {
        Default::default()
    } else {
        let let_stmt = if has_optional {
            quote! { let mut rhs = rhs.take(); }
        } else {
            quote! { let rhs = rhs.take(); }
        };
        let patch = quote! {
            pub fn patch(&mut self, rhs: &mut #opt_name) {
                #let_stmt
                #(#upts)*
            }
        };
        let into_opt = quote! {
            pub const fn into_opt(self) -> #opt_name {
                #opt_name { #(#into),* }
            }
        };
        let is_modified = quote! {
            pub const fn is_modified(&self) -> bool {
                #(#mods)||*
            }
        };
        let take = quote! {
            pub const fn take(&mut self) -> Self {
                Self { #(#take),* }
            }
        };
        (
            quote! {
                #patch
                #into_opt
            },
            quote! {
                #is_modified
                #take
            },
        )
    };

    #[cfg(feature = "bytemuck")]
    let impl_name_id = quote! {
        pub const ID: u8 = #id_og;
    };
    #[cfg(not(feature = "bytemuck"))]
    let impl_name_id = quote! {};

    #[cfg(feature = "bytemuck")]
    let impl_name = quote! {
        pub const UNPADDED_SIZE: usize = #(#size)+*;

        #impl_name
        #serde_og
    };
    let impl_name = quote! {
        impl #name {
            #impl_name_id
            #impl_name
        }
    };

    #[cfg(feature = "bytemuck")]
    let impl_opt_id = quote! {
        pub const ID: u8 = #id_opt;
    };
    #[cfg(not(feature = "bytemuck"))]
    let impl_opt_id = quote! {};

    #[cfg(feature = "bytemuck")]
    let impl_name_opt = quote! {
        pub const UNPADDED_SIZE: usize = #(#size)+*;

        #impl_name_opt
        #serde_opt
    };
    let impl_name_opt = quote! {
        impl #opt_name {
            #impl_opt_id
            #impl_name_opt
        }
    };

    quote! {
        #structure
        #impl_name
        #impl_name_opt
    }
    .into()
}
