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
            type Size = usize;
            fn try_from_ctx(src: &'a [u8], ctx: ::scroll::Endian) -> ::std::result::Result<(Self, Self::Size), Self::Error> {
                use ::scroll::Pread;
                let mut offset = &mut 0;
                let data  = #name { #(#items,)* };
                Ok((data, *offset))
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
            type Size = usize;
            fn try_into_ctx(self, mut dst: &mut [u8], ctx: ::scroll::Endian) -> ::std::result::Result<Self::Size, Self::Error> {
                use ::scroll::Pwrite;
                let mut offset = &mut 0;
                #(#items;)*;
                Ok(*offset)
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
    quote! {
        impl ::scroll::ctx::SizeWith<::scroll::Endian> for #name {
            type Units = usize;
            #[inline]
            fn size_with(_ctx: &::scroll::Endian) -> Self::Units {
                ::std::mem::size_of::<#name>()
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

fn impl_cread_struct(name: &syn::Ident, fields: &[syn::Field]) -> quote::Tokens {
    let items: Vec<_> = fields.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        match ty {
            &syn::Ty::Array(ref arrty, ref constexpr) => {
                match constexpr {
                    &syn::ConstExpr::Lit(syn::Lit::Int(size, _)) => {
                        let incr = quote! { ::std::mem::size_of::<#arrty>() };
                        quote! {
                            #ident: {
                                let mut __tmp: #ty = [0; #size as usize];
                                for i in 0..__tmp.len() {
                                    __tmp[i] = src.cread_with(*offset, ctx);
                                    *offset += #incr;
                                }
                                __tmp
                            }
                        }
                    },
                    _ => panic!("IOread derive with bad array constexpr")
                }
            },
            _ => {
                let size = quote! { ::std::mem::size_of::<#ty>() };
                quote! {
                    #ident: { let res = src.cread_with::<#ty>(*offset, ctx); *offset += #size; res }
                }
            }
        }
    }).collect();

    quote! {
        impl<'a> ::scroll::ctx::FromCtx<::scroll::Endian> for #name where #name: 'a {
            fn from_ctx(src: &[u8], ctx: ::scroll::Endian) -> Self {
                use ::scroll::Cread;
                let mut offset = &mut 0;
                let data = #name { #(#items,)* };
                data
            }
        }
    }
}

fn impl_from_ctx(ast: &syn::MacroInput) -> quote::Tokens {
    let name = &ast.ident;
    match &ast.body {
        &syn::Body::Struct(ref data) => {
            match data {
                &syn::VariantData::Struct(ref fields) => {
                    impl_cread_struct(name, &fields)
                },
                _ => {
                    panic!("IOread can only be derived for a regular struct with public fields")
                }
            }
        },
        _ => panic!("IOread can only be derived for structs")
    }
}

#[proc_macro_derive(IOread)]
pub fn derive_ioread(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_macro_input(&s).unwrap();
    let gen = impl_from_ctx(&ast);
    gen.parse().unwrap()
}

fn impl_into_ctx(name: &syn::Ident, fields: &[syn::Field]) -> quote::Tokens {
    let items: Vec<_> = fields.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        let size = quote! { ::std::mem::size_of::<#ty>() };
        match ty {
            &syn::Ty::Array(ref arrty, _) => {
                quote! {
                    let size = ::std::mem::size_of::<#arrty>();
                    for i in 0..self.#ident.len() {
                        dst.cwrite_with(self.#ident[i], *offset, ctx);
                        *offset += size;
                    }
                }
            },
            _ => {
                quote! {
                    dst.cwrite_with(self.#ident, *offset, ctx);
                    *offset += #size;
                }
            }
        }
    }).collect();

    quote! {
        impl ::scroll::ctx::IntoCtx<::scroll::Endian> for #name {
            fn into_ctx(self, mut dst: &mut [u8], ctx: ::scroll::Endian) {
                use ::scroll::Cwrite;
                let mut offset = &mut 0;
                #(#items;)*;
                ()
            }
        }
    }
}

fn impl_iowrite(ast: &syn::MacroInput) -> quote::Tokens {
    let name = &ast.ident;
    match &ast.body {
        &syn::Body::Struct(ref data) => {
            match data {
                &syn::VariantData::Struct(ref fields) => {
                    impl_into_ctx(name, &fields)
                },
                _ => {
                    panic!("IOwrite can only be derived for a regular struct with public fields")
                }
            }
        },
        _ => panic!("IOwrite can only be derived for structs")
    }
}

#[proc_macro_derive(IOwrite)]
pub fn derive_iowrite(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_macro_input(&s).unwrap();
    let gen = impl_iowrite(&ast);
    gen.parse().unwrap()
}
