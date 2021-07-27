use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use proc_macro_error::{abort_call_site, proc_macro_error, ResultExt};
use quote::{quote, ToTokens};
use syn::{self, ext::IdentExt, DataEnum, DataStruct, DeriveInput, Fields, Variant};
use to_vec::ToVec;

fn find_attr_fields<'a>(fields: &'a Fields, name: &str) -> Option<&'a Ident> {
    let fields = fields
        .iter()
        .filter(|f| {
            f.attrs.iter().any(|a| match a.path.get_ident() {
                None => false,
                Some(ident) => ident.unraw().to_string().eq(name),
            })
        })
        .to_vec();
    match fields.len() {
        0 => None,
        1 => fields[0].ident.as_ref(),
        _ => abort_call_site!(format!("Multiple fields found with attribute #[{}]", name)),
    }
}

#[proc_macro_derive(MIDIEvent, attributes(key, channel, delta))]
#[proc_macro_error]
pub fn midi_event(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).expect_or_abort("Couldn't parse for MIDIEvent");

    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Is it a struct?
    if let syn::Data::Struct(DataStruct { ref fields, .. }) = ast.data {
        let delta_field = find_attr_fields(fields, "delta");
        let key_field = find_attr_fields(fields, "key");
        let channel_field = find_attr_fields(fields, "channel");

        let delta_ident = match delta_field {
            None => abort_call_site!("MIDI events must have a delta time field (use #[delta])!"),
            Some(ident) => ident,
        };

        if key_field.is_some() && channel_field.is_none() {
            abort_call_site!(
                "Key events must also have a channel (use #[channel] along with #[key])!"
            );
        }

        // fields.iter().last().unwrap().attrs[0].
        let mut generated_impl = Vec::new();
        let mut generated_trait_impl = Vec::new();
        let mut generated_traits = Vec::new();

        generated_trait_impl.push(quote! {
            #[inline(always)]
            fn delta(&self) -> D {
                self.#delta_ident
            }

            #[inline(always)]
            fn delta_mut(&mut self) -> &mut D {
                &mut self.#delta_ident
            }
        });

        match key_field {
            None => {
                generated_impl.push(quote! {
                    #[inline(always)]
                    fn key(&self) -> Option<u8> {
                        None
                    }

                    #[inline(always)]
                    fn key_mut(&mut self) -> Option<&mut u8> {
                        None
                    }
                });

                generated_trait_impl.push(quote! {
                    #[inline(always)]
                    fn key(&self) -> Option<u8> {
                        None
                    }

                    #[inline(always)]
                    fn key_mut(&mut self) -> Option<&mut u8> {
                        None
                    }

                    #[inline(always)]
                    fn as_key_event<'a>(&'a self) -> Option<Box<&'a dyn KeyEvent>> {
                        None
                    }

                    #[inline(always)]
                    fn as_key_event_mut<'a>(&'a mut self) -> Option<Box<&'a mut dyn KeyEvent>> {
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

                    #[inline(always)]
                    fn as_key_event<'a>(&'a self) -> Box<&'a dyn KeyEvent> {
                        Box::new(self)
                    }

                    #[inline(always)]
                    fn as_key_event_mut<'a>(&'a mut self) -> Box<&'a mut dyn KeyEvent> {
                        Box::new(self)
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

                    #[inline(always)]
                    fn as_key_event<'a>(&'a self) -> Option<Box<&'a dyn KeyEvent>> {
                        Some(Box::new(self))
                    }

                    #[inline(always)]
                    fn as_key_event_mut<'a>(&'a mut self) -> Option<Box<&'a mut dyn KeyEvent>> {
                        Some(Box::new(self))
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
                generated_impl.push(quote! {
                    #[inline(always)]
                    fn channel(&self) -> Option<u8> {
                        None
                    }

                    #[inline(always)]
                    fn channel_mut(&mut self) -> Option<&mut u8> {
                        None
                    }
                });

                generated_trait_impl.push(quote! {
                    #[inline(always)]
                    fn channel(&self) -> Option<u8> {
                        None
                    }

                    #[inline(always)]
                    fn channel_mut(&mut self) -> Option<&mut u8> {
                        None
                    }

                    #[inline(always)]
                    fn as_channel_event<'a>(&'a self) -> Option<Box<&'a dyn ChannelEvent>> {
                        None
                    }

                    #[inline(always)]
                    fn as_channel_event_mut<'a>(&'a mut self) -> Option<Box<&'a mut dyn ChannelEvent>> {
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

                    #[inline(always)]
                    fn as_channel_event<'a>(&'a self) -> Box<&'a dyn ChannelEvent> {
                        Box::new(self)
                    }

                    #[inline(always)]
                    fn as_channel_event_mut<'a>(&'a mut self) -> Box<&'a mut dyn ChannelEvent> {
                        Box::new(self)
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

                    #[inline(always)]
                    fn as_channel_event<'a>(&'a self) -> Option<Box<&'a dyn ChannelEvent>> {
                        Some(Box::new(self))
                    }

                    #[inline(always)]
                    fn as_channel_event_mut<'a>(&'a mut self) -> Option<Box<&'a mut dyn ChannelEvent>> {
                        Some(Box::new(self))
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

            impl<D: MIDINum> MIDIEvent<D> for #name <D> #where_clause {
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

#[proc_macro_derive(CastEventDelta, attributes(delta))]
#[proc_macro_error]
pub fn cast_event_delta(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).expect_or_abort("Couldn't parse for CastEventDelta");

    let name = &ast.ident;

    // Is it a struct?
    if let syn::Data::Struct(DataStruct { ref fields, .. }) = ast.data {
        let delta_field = find_attr_fields(fields, "delta");

        let delta_ident = match delta_field {
            None => abort_call_site!(
                "Struct must have a marked delta field for CastEventDelta (use #[delta])!"
            ),
            Some(ident) => ident,
        };

        let mut generated_cast = Vec::new();

        for field in fields.iter() {
            match &field.ident {
                None => {}
                Some(ident) => {
                    if ident != delta_ident {
                        generated_cast.push(quote! {
                            #ident: self.#ident.clone(),
                        });
                    }
                }
            }
        }

        let gen = quote! {
            impl<TT, DT: MIDINum> CastEventDelta<DT, #name <DT>> for #name <TT>
            where
                TT: MIDINum + MIDINumInto<DT>,
            {
                #[inline(always)]
                fn cast_delta(&self) -> #name <DT> {
                    #name {
                        #(#generated_cast)*
                        #delta_ident: self.#delta_ident.midi_num_into(),
                    }
                }
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
    let event_ident = Ident::new(event_name, name.span());
    event_ident
}

fn event_struct_from_enum(name: &Ident) -> Ident {
    let event_name = name.unraw().to_string();
    let event_name = event_name + "Event";
    let event_ident = Ident::new(&event_name[..], name.span());
    event_ident
}

#[proc_macro_derive(NewEvent)]
#[proc_macro_error]
pub fn create_new_event(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).expect_or_abort("Couldn't parse for NewEvent");

    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Is it a struct?
    if let syn::Data::Struct(DataStruct { ref fields, .. }) = ast.data {
        let mut new_args = Vec::new();
        let mut assign = Vec::new();

        let event_ident = event_enum_from_struct(name);
        let snake_case = name.unraw().to_string()[..].to_case(Case::Snake);
        let new_ident = Ident::new(&format!("new_{}", snake_case)[..], Span::call_site());

        let doc_str = &format!("Creates a new `{}`.", name);
        let doc_str2 = &format!(
            "Creates a new `{}` wrapped in `Event::{}`.",
            name,
            event_ident.unraw().to_string()
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
            impl #impl_generics #name #ty_generics #where_clause {
                #[doc=#doc_str]
                pub fn new(#(#new_args)*) -> Self {
                    Self {
                        #(#assign)*
                    }
                }
            }

            impl<D: MIDINum> Event<D> {
                #[doc=#doc_str2]
                pub fn #new_ident(#(#new_args)*) -> Event::#ty_generics {
                    (#name :: new(#(#assign)*)).as_event()
                }
            }
        };

        gen.into()
    } else {
        // Nope. This is an Enum.
        abort_call_site!("#[derive(MIDIEvent)] is only defined for structs, not for enums!");
    }
}

#[proc_macro_derive(EventImpl, attributes(channel, key))]
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
            let field = variant.fields.iter().nth(0).unwrap();
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
            fn wrap(&self, tokens: TokenStream2) -> TokenStream2 {
                tokens
            }
        }

        fn create_match<T: Fn((&Variant, &Box<dyn Mapper>, &Box<dyn Mapper>)) -> TokenStream2>(
            variants: &Vec<&Variant>,
            map: T,
        ) -> TokenStream2 {
            let wrap_some: Box<dyn Mapper> = Box::new(WrapSome);
            let dont_wrap: Box<dyn Mapper> = Box::new(DontWrap);

            match_all(
                variants
                    .iter()
                    .map(|v| {
                        (
                            *v,
                            if is_key(v) { &wrap_some } else { &dont_wrap },
                            if is_channel(v) {
                                &wrap_some
                            } else {
                                &dont_wrap
                            },
                        )
                    })
                    .map(map)
                    .to_vec(),
            )
        }

        let variants = variants.iter().to_vec();

        let clone_match = create_match(&variants, |v| {
            let ident = &v.0.ident;
            if is_boxed(v.0) {
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
                create_match(&variants, |(v, _, _)| make_match_line(&v.ident, $res))
            };
            (key, $res:expr) => {
                create_match(&variants, |(v, wrap_key, _)| {
                    make_match_line(&v.ident, wrap_key.wrap($res))
                })
            };
            (channel, $res:expr) => {
                create_match(&variants, |(v, _, wrap_chan)| {
                    make_match_line(&v.ident, wrap_chan.wrap($res))
                })
            };
        }

        let delta = make_map!(quote! { event.delta });
        let delta_mut = make_map!(quote! { &mut event.delta });
        let key = make_map!(key, quote! { event.key() });
        let key_mut = make_map!(key, quote! { event.key_mut() });
        let channel = make_map!(channel, quote! { event.channel() });
        let channel_mut = make_map!(channel, quote! { event.channel_mut() });
        let as_key_event = make_map!(quote! { event.as_key_event() });
        let as_key_event_mut = make_map!(quote! { event.as_key_event_mut() });
        let as_channel_event = make_map!(quote! { event.as_channel_event() });
        let as_channel_event_mut = make_map!(quote! { event.as_channel_event_mut() });

        let cast_delta = make_map!(quote! { event.cast_delta().as_event() });

        let mut event_wrap_impl = Vec::new();
        for variant in variants.iter() {
            let ident = &variant.ident;
            let struct_ident = event_struct_from_enum(ident);
            let doc_str = &format!(
                "Wraps the `{}` in a `Event::{}`.",
                struct_ident.unraw().to_string(),
                ident.unraw().to_string()
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
                fn delta(&self) -> D {
                    #delta
                }

                #[inline(always)]
                fn delta_mut(&mut self) -> &mut D {
                    #delta_mut
                }

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
                fn as_key_event<'a>(&'a self) -> Option<Box<&'a dyn KeyEvent>> {
                    #as_key_event
                }

                #[inline(always)]
                fn as_key_event_mut<'a>(&'a mut self) -> Option<Box<&'a mut dyn KeyEvent>> {
                    #as_key_event_mut
                }

                #[inline(always)]
                fn as_channel_event<'a>(&'a self) -> Option<Box<&'a dyn ChannelEvent>> {
                    #as_channel_event
                }

                #[inline(always)]
                fn as_channel_event_mut<'a>(&'a mut self) -> Option<Box<&'a mut dyn ChannelEvent>> {
                    #as_channel_event_mut
                }
            }

            #(#event_wrap_impl)*

            impl<TT, DT: MIDINum> CastEventDelta<DT, #name<DT>> for #name<TT>
            where
                TT: MIDINum + MIDINumInto<DT>,
            {
                #[inline(always)]
                fn cast_delta(&self) -> #name<DT> {
                    #cast_delta
                }
            }
        };

        gen.into()
    } else {
        // Nope. This is an Enum.
        abort_call_site!("#[derive(MIDIEvent)] is only defined for structs, not for enums!");
    }
}
