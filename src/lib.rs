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
        match ty {
            &syn::Ty::Array(_, ref constexpr) => {
                match constexpr {
                    &syn::ConstExpr::Lit(syn::Lit::Int(size, _)) => {
                        quote! {
                            #ident: { let mut __tmp: #ty = [0; #size as usize]; src.gread_inout_with(offset, &mut __tmp, ctx)?; __tmp }
                        }
                    },
                    _ => panic!("Pread derive with bad array constexpr")
                }
            },
            _ => {
                quote! {
                    #ident: src.gread_with::<#ty>(offset, ctx)?
                }
            }
        }
    }).collect();
    
    quote! {
        impl<'a> ::scroll::ctx::TryFromCtx<'a, ::scroll::Endian> for #name where #name: 'a {
            type Error = ::scroll::Error;
            fn try_from_ctx(src: &'a [u8], ctx: ::scroll::Endian) -> ::std::result::Result<Self, Self::Error> {
                use ::scroll::Gread;
                let mut offset = &mut 0;
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
        let ty = &f.ty;
        match ty {
            &syn::Ty::Array(_, _) => {
                quote! {
                    for i in 0..self.#ident.len() {
                        dst.gwrite_with(self.#ident[i], offset, ctx)?;
                    }
                }
            },
            _ => {
                quote! {
                    dst.gwrite_with(self.#ident, offset, ctx)?
                }
            }
        }
    }).collect();
    
    quote! {
        impl ::scroll::ctx::TryIntoCtx<::scroll::Endian> for #name {
            type Error = ::scroll::Error;
            fn try_into_ctx(self, mut dst: &mut [u8], ctx: ::scroll::Endian) -> ::std::result::Result<(), Self::Error> {
                use ::scroll::Gwrite;
                let mut offset = &mut 0;
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

fn size_with(name: &syn::Ident) -> quote::Tokens {
    let size = quote! { ::std::mem::size_of::<#name>() };
    quote! {
        impl ::scroll::ctx::SizeWith<::scroll::Endian> for #name {
            type Units = usize;
            #[inline]
            fn size_with(_ctx: &::scroll::Endian) -> Self::Units {
                #size
            }
        }
    }
}

fn impl_size_with(ast: &syn::MacroInput) -> quote::Tokens {
    let name = &ast.ident;
    match &ast.body {
        &syn::Body::Struct(ref data) => {
            match data {
                &syn::VariantData::Struct(_) => {
                    size_with(name)
                },
                _ => {
                    panic!("SizeWith can only be derived for a regular struct with public fields")
                }
            }
        },
        _ => panic!("SizeWith can only be derived for structs")
    }
}

#[proc_macro_derive(SizeWith)]
pub fn derive_sizewith(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_macro_input(&s).unwrap();
    let gen = impl_size_with(&ast);
    gen.parse().unwrap()
}
