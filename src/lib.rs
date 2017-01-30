#![recursion_limit="1024"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

fn impl_struct(name: &syn::Ident, fields: &[syn::Field]) -> quote::Tokens {
    let items: Vec<_> = fields.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        quote! {
            #ident: src.gread_with::<#ty>(offset, ctx)?
        }
    }).collect();
    
    quote! {
        impl<'a> ::scroll::ctx::TryFromCtx<'a> for #name where #name: 'a {
            type Error = ::scroll::Error;
            fn try_from_ctx(src: &'a [u8], (mut offset, ctx): (usize, ::scroll::Endian)) -> ::std::result::Result<Self, Self::Error> {
                use ::scroll::Gread;
                let mut offset = &mut offset;
                let data  = #name { #(#items,)* };
                Ok(data)
            }
        }
    }
}

fn impl_try_from_ctx(ast: &syn::MacroInput) -> quote::Tokens {
    let name = &ast.ident;
    match &ast.body {
        &syn::Body::Struct(ref data) => {
            match data {
                &syn::VariantData::Struct(ref fields) => {
                    impl_struct(name, &fields)
                },
                _ => {
                    panic!("Pread can only be derived for a regular struct with public fields")
                }
            }
        },
        _ => panic!("Pread can only be derived for structs")
    }
}

#[proc_macro_derive(Pread)]
pub fn derive_pread(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_macro_input(&s).unwrap();
    let gen = impl_try_from_ctx(&ast);
    gen.parse().unwrap()
}

fn impl_try_into_ctx(name: &syn::Ident, fields: &[syn::Field]) -> quote::Tokens {
    let items: Vec<_> = fields.iter().map(|f| {
        let ident = &f.ident;
        quote! {
            dst.gwrite_with(self.#ident, offset, ctx)?
        }
    }).collect();
    
    quote! {
        impl ::scroll::ctx::TryIntoCtx for #name {
            type Error = ::scroll::Error;
            fn try_into_ctx(self, mut dst: &mut [u8], (mut offset, ctx): (usize, ::scroll::Endian)) -> ::std::result::Result<(), Self::Error> {
                use ::scroll::Gwrite;
                let mut offset = &mut offset;
                #(#items;)*;
                Ok(())
            }
        }
    }
}

fn impl_pwrite(ast: &syn::MacroInput) -> quote::Tokens {
    let name = &ast.ident;
    match &ast.body {
        &syn::Body::Struct(ref data) => {
            match data {
                &syn::VariantData::Struct(ref fields) => {
                    impl_try_into_ctx(name, &fields)
                },
                _ => {
                    panic!("Pwrite can only be derived for a regular struct with public fields")
                }
            }
        },
        _ => panic!("Pwrite can only be derived for structs")
    }
}

#[proc_macro_derive(Pwrite)]
pub fn derive_pwrite(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_macro_input(&s).unwrap();
    let gen = impl_pwrite(&ast);
    gen.parse().unwrap()
}
