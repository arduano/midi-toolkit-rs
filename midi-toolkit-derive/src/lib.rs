use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use proc_macro_error::{abort_call_site, proc_macro_error, ResultExt};
use quote::{quote, ToTokens};
use syn::{self, ext::IdentExt, Attribute, DataEnum, DataStruct, DeriveInput, Fields, Variant};

fn has_attr(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|a| match a.path.get_ident() {
        None => false,
        Some(ident) => ident.unraw().to_string().eq(name),
    })
}

fn find_attr_fields<'a>(fields: &'a Fields, name: &str) -> Option<&'a Ident> {
    let fields = fields
        .iter()
        .filter(|f| has_attr(&f.attrs, name))
        .collect::<Vec<_>>();
    match fields.len() {
        0 => None,
        1 => fields[0].ident.as_ref(),
        _ => abort_call_site!(format!("Multiple fields found with attribute #[{name}]")),
    }
}

#[proc_macro_derive(MIDIEvent, attributes(key, channel, playback))]
#[proc_macro_error]
pub fn midi_event(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).expect_or_abort("Couldn't parse for MIDIEvent");

    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Is it a struct?
    if let syn::Data::Struct(DataStruct { ref fields, .. }) = ast.data {
        let key_field = find_attr_fields(fields, "key");
        let channel_field = find_attr_fields(fields, "channel");

        let playback_event = has_attr(&ast.attrs, "playback");

        if key_field.is_some() && channel_field.is_none() {
            abort_call_site!(
                "Key events must also have a channel (use #[channel] along with #[key])!"
            );
        }

        let mut generated_impl = Vec::new();
        let mut generated_trait_impl = Vec::new();
        let mut generated_traits = Vec::new();

        if playback_event {
            generated_impl.push(quote! {
                #[inline(always)]
                pub fn as_u32(&self) -> u32 {
                    PlaybackEvent::as_u32(self)
                }
            });

            generated_trait_impl.push(quote! {
                #[inline(always)]
                fn as_u32(&self) -> Option<u32> {
                    Some(PlaybackEvent::as_u32(self))
                }
            });
        } else {
            generated_trait_impl.push(quote! {
                #[inline(always)]
                fn as_u32(&self) -> Option<u32> {
                    None
                }
            });
        }

        match key_field {
            None => {
                generated_trait_impl.push(quote! {
                    #[inline(always)]
                    fn key(&self) -> Option<u8> {
                        None
                    }

                    #[inline(always)]
                    fn key_mut(&mut self) -> Option<&mut u8> {
                        None
                    }
                });
            }
            Some(ident) => {
                generated_impl.push(quote! {
                    #[inline(always)]
                    pub fn key(&self) -> u8 {
                        self.#ident
                    }

                    #[inline(always)]
                    pub fn key_mut(&mut self) -> &mut u8 {
                        &mut self.#ident
                    }
                });

                generated_trait_impl.push(quote! {
                    #[inline(always)]
                    fn key(&self) -> Option<u8> {
                        Some(self.#ident)
                    }

                    #[inline(always)]
                    fn key_mut(&mut self) -> Option<&mut u8> {
                        Some(&mut self.#ident)
                    }
                });

                generated_traits.push(quote! {
                    impl #impl_generics KeyEvent for #name #ty_generics #where_clause {
                        #[inline(always)]
                        fn key(&self) -> u8 {
                            self.#ident
                        }

                        #[inline(always)]
                        fn key_mut(&mut self) -> &mut u8 {
                            &mut self.#ident
                        }
                    }
                });
            }
        }

        match channel_field {
            None => {
                generated_trait_impl.push(quote! {
                    #[inline(always)]
                    fn channel(&self) -> Option<u8> {
                        None
                    }

                    #[inline(always)]
                    fn channel_mut(&mut self) -> Option<&mut u8> {
                        None
                    }
                });
            }
            Some(ident) => {
                generated_impl.push(quote! {
                    #[inline(always)]
                    pub fn channel(&self) -> u8 {
                        self.#ident
                    }

                    #[inline(always)]
                    pub fn channel_mut(&mut self) -> &mut u8 {
                        &mut self.#ident
                    }
                });

                generated_trait_impl.push(quote! {
                    #[inline(always)]
                    fn channel(&self) -> Option<u8> {
                        Some(self.#ident)
                    }

                    #[inline(always)]
                    fn channel_mut(&mut self) -> Option<&mut u8> {
                        Some(&mut self.#ident)
                    }
                });

                generated_traits.push(quote! {
                    impl #impl_generics ChannelEvent for #name #ty_generics #where_clause {
                        #[inline(always)]
                        fn channel(&self) -> u8 {
                            self.#ident
                        }

                        #[inline(always)]
                        fn channel_mut(&mut self) -> &mut u8 {
                            &mut self.#ident
                        }
                    }
                });
            }
        }

        let gen = quote! {
            #(#generated_traits)*

            impl MIDIEvent for #name #where_clause {
                #(#generated_trait_impl)*
            }

            impl #impl_generics #name #ty_generics #where_clause {
                #(#generated_impl)*
            }
        };

        gen.into()
    } else {
        // Nope. This is an Enum.
        abort_call_site!("#[derive(MIDIEvent)] is only defined for structs, not for enums!");
    }
}

fn event_enum_from_struct(name: &Ident) -> Ident {
    let event_name = name.unraw().to_string();
    let event_name = &event_name[..event_name.len() - 5];
    Ident::new(event_name, name.span())
}

fn event_struct_from_enum(name: &Ident) -> Ident {
    let event_name = name.unraw().to_string();
    let event_name = event_name + "Event";
    Ident::new(&event_name[..], name.span())
}

#[proc_macro_derive(NewEvent)]
#[proc_macro_error]
pub fn create_new_event(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).expect_or_abort("Couldn't parse for NewEvent");

    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, _ty_generics, where_clause) = generics.split_for_impl();

    // Is it a struct?
    if let syn::Data::Struct(DataStruct { ref fields, .. }) = ast.data {
        let mut new_args = Vec::new();
        let mut assign = Vec::new();

        let event_ident = event_enum_from_struct(name);
        let snake_case = name.unraw().to_string()[..].to_case(Case::Snake);
        let new_ident = Ident::new(&format!("new_{snake_case}")[..], Span::call_site());
        let new_delta_ident = Ident::new(&format!("new_delta_{snake_case}")[..], Span::call_site());

        let doc_str = &format!("Creates a new `{name}`.");
        let doc_str2 = &format!(
            "Creates a new [`{name}`](crate::events::{name}) wrapped in [`Event::{ident}`](crate::events::Event::{ident}).",
            ident = event_ident.unraw(),
        );
        let doc_str2_delta = &format!(
            "Creates a new [`{name}`](crate::events::{name}) wrapped in [`Event::{ident}`](crate::events::Event::{ident}).",
            ident = event_ident.unraw(),
        );

        for field in fields.iter() {
            match &field.ident {
                None => {}
                Some(ident) => {
                    let ty = &field.ty;
                    new_args.push(quote! {#ident: #ty,});
                    assign.push(quote! {#ident,});
                }
            }
        }

        let gen = quote! {
            impl #impl_generics #name #where_clause {
                #[doc=#doc_str]
                #[inline(always)]
                pub fn new(#(#new_args)*) -> Self {
                    Self {
                        #(#assign)*
                    }
                }
            }

            impl Event {
                #[doc=#doc_str2]
                #[inline(always)]
                pub fn #new_ident(#(#new_args)*) -> Event {
                    (#name :: new(#(#assign)*)).as_event()
                }

                #[doc=#doc_str2_delta]
                #[inline(always)]
                pub fn #new_delta_ident<D: MIDINum>(delta: D, #(#new_args)*) -> Delta<D, Event> {
                    Delta::new(delta, (#name :: new(#(#assign)*)).as_event())
                }
            }
        };

        gen.into()
    } else {
        // Nope. This is an Enum.
        abort_call_site!("#[derive(MIDIEvent)] is only defined for structs, not for enums!");
    }
}

#[proc_macro_derive(EventImpl, attributes(channel, key, playback, tempo))]
#[proc_macro_error]
pub fn event_impl(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).expect_or_abort("Couldn't parse for EventImpl");

    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Is it a struct?
    if let syn::Data::Enum(DataEnum { variants, .. }) = ast.data {
        fn has_attr(v: &Variant, name: &str) -> bool {
            v.attrs.iter().any(|a| match a.path.get_ident() {
                None => false,
                Some(ident) => ident.unraw().to_string().eq(name),
            })
        }
        fn is_key(v: &Variant) -> bool {
            has_attr(v, "key")
        }
        fn is_channel(v: &Variant) -> bool {
            has_attr(v, "channel")
        }
        fn is_playback(v: &Variant) -> bool {
            has_attr(v, "playback")
        }

        fn match_all(lines: Vec<TokenStream2>) -> TokenStream2 {
            quote! {
                match self {
                    #(#lines)*
                }
            }
        }

        fn make_match_line(ident: &Ident, res: TokenStream2) -> TokenStream2 {
            quote! {
                Event::#ident(event) => #res,
            }
        }

        fn is_boxed(variant: &Variant) -> bool {
            let field = variant.fields.iter().next().unwrap();
            let mut tokens = TokenStream2::new();
            field.ty.to_tokens(&mut tokens);
            tokens.to_string().starts_with("Box <")
        }

        trait Mapper {
            fn wrap(&self, tokens: TokenStream2) -> TokenStream2;
        }

        struct WrapSome;
        impl Mapper for WrapSome {
            fn wrap(&self, tokens: TokenStream2) -> TokenStream2 {
                quote! { Some(#tokens) }
            }
        }

        struct DontWrap;
        impl Mapper for DontWrap {
            fn wrap(&self, _: TokenStream2) -> TokenStream2 {
                quote! { None }
            }
        }

        struct Mappers<'a> {
            wrap_key: &'a dyn Mapper,
            wrap_channel: &'a dyn Mapper,
            wrap_playback: &'a dyn Mapper,
        }

        fn create_match<T: Fn(&Variant, Mappers) -> TokenStream2>(
            variants: &[&Variant],
            map: T,
        ) -> TokenStream2 {
            let wrap_some: Box<dyn Mapper> = Box::new(WrapSome);
            let dont_wrap: Box<dyn Mapper> = Box::new(DontWrap);

            match_all(
                variants
                    .iter()
                    .map(|v| {
                        map(
                            v,
                            Mappers {
                                wrap_key: if is_key(v) { &*wrap_some } else { &*dont_wrap },
                                wrap_channel: if is_channel(v) {
                                    &*wrap_some
                                } else {
                                    &*dont_wrap
                                },
                                wrap_playback: if is_playback(v) {
                                    &*wrap_some
                                } else {
                                    &*dont_wrap
                                },
                            },
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        }

        let variants = variants.iter().collect::<Vec<_>>();

        let clone_match = create_match(&variants, |v, _mappers| {
            let ident = &v.ident;
            if is_boxed(v) {
                make_match_line(
                    ident,
                    quote! { Event::#ident(Box::new(event.as_ref().clone())) },
                )
            } else {
                make_match_line(ident, quote! { Event::#ident(event.clone()) })
            }
        });

        macro_rules! make_map {
            ($res:expr) => {
                create_match(&variants, |v, _| make_match_line(&v.ident, $res))
            };
            (key, $res:expr) => {
                create_match(&variants, |v, Mappers { wrap_key, .. }| {
                    make_match_line(&v.ident, wrap_key.wrap($res))
                })
            };
            (channel, $res:expr) => {
                create_match(&variants, |v, Mappers { wrap_channel, .. }| {
                    make_match_line(&v.ident, wrap_channel.wrap($res))
                })
            };
            (playback, $res:expr) => {
                create_match(&variants, |v, Mappers { wrap_playback, .. }| {
                    make_match_line(&v.ident, wrap_playback.wrap($res))
                })
            };
        }

        let key = make_map!(key, quote! { event.key() });
        let key_mut = make_map!(key, quote! { event.key_mut() });
        let channel = make_map!(channel, quote! { event.channel() });
        let channel_mut = make_map!(channel, quote! { event.channel_mut() });
        let as_u32 = make_map!(playback, quote! { event.as_u32() });

        let serialize_event = make_map!(quote! { event.serialize_event(buf) });

        let mut event_wrap_impl = Vec::new();
        for variant in variants.iter() {
            let ident = &variant.ident;
            let struct_ident = event_struct_from_enum(ident);
            let doc_str = &format!(
                "Wraps the `{}` in a `Event::{}`.",
                struct_ident.unraw(),
                ident.unraw()
            );
            if is_boxed(variant) {
                event_wrap_impl.push(quote! {
                    impl #impl_generics #struct_ident #ty_generics {
                        #[doc=#doc_str]
                        #[inline(always)]
                        pub fn as_event(self) -> #name #ty_generics {
                            #name::#ident(Box::new(self))
                        }
                    }
                });
            } else {
                event_wrap_impl.push(quote! {
                    impl #impl_generics #struct_ident #ty_generics {
                        #[doc=#doc_str]
                        #[inline(always)]
                        pub fn as_event(self) -> #name #ty_generics {
                            #name::#ident(self)
                        }
                    }
                });
            }
        }

        let gen = quote! {
            impl #impl_generics Clone for #name #ty_generics #where_clause {
                #[inline(always)]
                fn clone(&self) -> #name #ty_generics {
                    #clone_match
                }
            }

            impl#impl_generics MIDIEvent #ty_generics for #name #ty_generics #where_clause {
                #[inline(always)]
                fn key(&self) -> Option<u8> {
                    #key
                }

                #[inline(always)]
                fn key_mut(&mut self) -> Option<&mut u8> {
                    #key_mut
                }

                #[inline(always)]
                fn channel(&self) -> Option<u8> {
                    #channel
                }

                #[inline(always)]
                fn channel_mut(&mut self) -> Option<&mut u8> {
                    #channel_mut
                }

                #[inline(always)]
                fn as_u32(&self) -> Option<u32> {
                    #as_u32
                }
            }

            #(#event_wrap_impl)*

            impl SerializeEvent for Event {
                #[inline(always)]
                fn serialize_event<T: Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
                    #serialize_event
                }
            }
        };

        gen.into()
    } else {
        // Nope. This is an Enum.
        abort_call_site!("#[derive(MIDIEvent)] is only defined for structs, not for enums!");
    }
}
