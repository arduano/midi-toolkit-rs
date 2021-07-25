use std::borrow::Borrow;

use num_traits::Num;
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use proc_macro_error::{abort, abort_call_site, proc_macro_error, ResultExt};
use quote::quote;
use syn::{
    self, ext::IdentExt, spanned::Spanned, DataStruct, DeriveInput, Field, Fields, Lit, Meta,
    MetaNameValue, Visibility,
};
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
            fn delta(&self) -> D {
                self.#delta_ident
            }

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
                        fn key(&self) -> u8 {
                            self.#ident
                        }

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
                        fn channel(&self) -> u8 {
                            self.#ident
                        }

                        fn channel_mut(&mut self) -> &mut u8 {
                            &mut self.#ident
                        }
                    }
                });
            }
        }

        let gen = quote! {
            #(#generated_traits)*

            impl<D: DeltaNum> MIDIEvent<D> for #name <D> #where_clause {
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
            impl<TT, DT: DeltaNum> CastEventDelta<DT, #name <DT>> for #name <TT>
            where
                TT: DeltaNum + DeltaNumInto<DT>,
            {
                fn clone(&self) -> Self {
                    #name {
                        #(#generated_cast)*
                        #delta_ident: self.#delta_ident.clone(),
                    }
                }
                fn cast_delta(&self) -> #name <DT> {
                    #name {
                        #(#generated_cast)*
                        #delta_ident: self.#delta_ident.delta_into(),
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
