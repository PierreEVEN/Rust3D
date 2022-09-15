extern crate proc_macro;

use proc_macro::{   TokenStream, TokenTree};
use std::str::FromStr;

use quote::{quote};
use syn::Data::{Enum, Struct};
use syn::DeriveInput;
use syn::parse_macro_input;

#[proc_macro_derive(OpsAdd)]
pub fn derive_operator_add(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let type_name = ast.ident;

    let mut expanded = quote! {};

    match ast.data {
        Struct(data_struct) => {
            let field_vector: Vec<&Option<syn::Ident>> = data_struct.fields.iter().map(|variant| &variant.ident).collect();

            expanded.extend(quote! {
                impl<T: Default + Copy + ops::Add<Output=T>> ops::Add<T> for #type_name<T> {                    
                    type Output = #type_name<T>;                    
                    fn add(self, rhs: T) -> Self::Output {
                        Self::Output
                        {
                            #(#field_vector: self.#field_vector + rhs,)*
                        }
                    }
                }
            });
            expanded.extend(quote! {
                impl<T: Default + Copy + ops::Add<Output=T>> ops::Add<#type_name<T>> for #type_name<T> {                    
                    type Output = #type_name<T>;                    
                    fn add(self, rhs: #type_name<T>) -> Self::Output {
                        Self::Output
                        {
                            #(#field_vector: self.#field_vector + rhs.#field_vector,)*
                        }
                    }
                }
            });
        }
        _ => ()
    }

    expanded.into()
}

#[proc_macro_derive(OpsMul)]
pub fn derive_operator_mul(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let type_name = ast.ident;

    let mut expanded = quote! {};

    match ast.data {
        Struct(data_struct) => {
            let field_vector: Vec<&Option<syn::Ident>> = data_struct.fields.iter().map(|variant| &variant.ident).collect();

            expanded.extend(quote! {
                impl<T: Default + Copy + ops::Mul<Output=T>> ops::Mul<T> for #type_name<T> {                    
                    type Output = #type_name<T>;                    
                    fn mul(self, rhs: T) -> Self::Output {
                        Self::Output
                        {
                            #(#field_vector: self.#field_vector * rhs,)*
                        }
                    }
                }
            });
            expanded.extend(quote! {
                impl<T: Default + Copy + ops::Mul<Output=T>> ops::Mul<#type_name<T>> for #type_name<T> {                    
                    type Output = #type_name<T>;                    
                    fn mul(self, rhs: #type_name<T>) -> Self::Output {
                        Self::Output
                        {
                            #(#field_vector: self.#field_vector * rhs.#field_vector,)*
                        }
                    }
                }
            });
        }
        _ => ()
    }

    expanded.into()
}

#[proc_macro_derive(OpsSub)]
pub fn derive_operator_sub(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let type_name = ast.ident;

    let mut expanded = quote! {};

    match ast.data {
        Struct(data_struct) => {
            let field_vector: Vec<&Option<syn::Ident>> = data_struct.fields.iter().map(|variant| &variant.ident).collect();

            expanded.extend(quote! {
                impl<T: Default + Copy + ops::Sub<Output=T>> ops::Sub<T> for #type_name<T> {                    
                    type Output = #type_name<T>;                    
                    fn sub(self, rhs: T) -> Self::Output {
                        Self::Output
                        {
                            #(#field_vector: self.#field_vector - rhs,)*
                        }
                    }
                }
            });
            expanded.extend(quote! {
                impl<T: Default + Copy + ops::Sub<Output=T>> ops::Sub<#type_name<T>> for #type_name<T> {                    
                    type Output = #type_name<T>;                    
                    fn sub(self, rhs: #type_name<T>) -> Self::Output {
                        Self::Output
                        {
                            #(#field_vector: self.#field_vector - rhs.#field_vector,)*
                        }
                    }
                }
            });
        }
        _ => ()
    }

    expanded.into()
}

#[proc_macro_derive(OpsDiv)]
pub fn derive_operator_div(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let type_name = ast.ident;

    let mut expanded = quote! {};

    match ast.data {
        Struct(data_struct) => {
            let field_vector: Vec<&Option<syn::Ident>> = data_struct.fields.iter().map(|variant| &variant.ident).collect();

            expanded.extend(quote! {
                impl<T: Default + Copy + ops::Div<Output=T>> ops::Div<T> for #type_name<T> {                    
                    type Output = #type_name<T>;                    
                    fn div(self, rhs: T) -> Self::Output {
                        Self::Output
                        {
                            #(#field_vector: self.#field_vector / rhs,)*
                        }
                    }
                }
            });
            expanded.extend(quote! {
                impl<T: Default + Copy + ops::Div<Output=T>> ops::Div<#type_name<T>> for #type_name<T> {                    
                    type Output = #type_name<T>;                    
                    fn div(self, rhs: #type_name<T>) -> Self::Output {
                        Self::Output
                        {
                            #(#field_vector: self.#field_vector / rhs.#field_vector,)*
                        }
                    }
                }
            });
        }
        _ => ()
    }

    expanded.into()
}

#[proc_macro_derive(DefaultConstruct)]
pub fn derive_default_construct(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let type_name = ast.ident;
    let mut expanded = quote! {};

    match ast.data {
        Struct(data_struct) => {
            let field_vector: Vec<&Option<syn::Ident>> = data_struct.fields.iter().map(|variant| &variant.ident).collect();

            expanded.extend(quote! {
                impl<T: Default> #type_name<T> {                      
                    pub fn new(#(#field_vector: T,)*) -> Self {
                        Self
                        {
                            #(#field_vector,)*
                        }
                    }
                }
            });
        }
        _ => ()
    }

    expanded.into()
}

#[proc_macro_derive(EnumToStr)]
pub fn enum_to_str(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let type_name = ast.ident;

    match ast.data {
        Enum(data_enum) => {
            let mut string = format!("impl std::fmt::Display for {type_name} {{
                    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {{
                        f.write_str(
                            match self {{");
            
            for variant in data_enum.variants {
                let attr_count = variant.fields.len();
                string += format!("{}::{}", type_name, variant.ident).as_str();
                if attr_count > 0 {
                    string += "(";
                    for i in 0..attr_count {
                        if i == attr_count - 1 { string += "_)"; } else { string += "_,"; }
                    }
                }

                string += format!(" => {{ \" {} \" }},\n", variant.ident).as_str();            
            }
            string += format!("}})}}}}").as_str();

            let tokens = match TokenStream::from_str(string.as_str()) {
                Ok(ts) => { ts }
                Err(_) => { panic!("failed to parse token") }
            };

            return tokens;
        }
        _ => {}
    };
    TokenStream::new()
}