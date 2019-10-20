#![cfg_attr(feature = "external_doc", feature(external_doc))]
#![cfg_attr(feature = "external_doc", doc(include = "../README.md"))]

extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use syn::parse::{Parse, ParseStream};
use syn::parse_macro_input;
use syn::spanned::Spanned;
use quote::quote;

struct Attributes {}

impl Parse for Attributes {
    fn parse( input: ParseStream ) -> Result< Self, syn::Error > {
        if !input.is_empty() {
            return Err( input.error( "unexpected attribute" ) );
        } else {
            Ok( Attributes {} )
        }
    }
}

fn create_test_attr() -> syn::Attribute {
    let test_ident = syn::Ident::new( "test", Span::call_site() );
    let mut test_segments = syn::punctuated::Punctuated::new();
    test_segments.push( syn::PathSegment {
        ident: test_ident,
        arguments: syn::PathArguments::None
    });

    syn::Attribute {
        pound_token: Default::default(),
        style: syn::AttrStyle::Outer,
        bracket_token: Default::default(),
        path: syn::Path {
            leading_colon: None,
            segments: test_segments
        },
        tokens: TokenStream::new()
    }
}

fn get_args( raw_args: &syn::punctuated::Punctuated< syn::FnArg, syn::token::Comma > ) -> Result< Vec< (&syn::Ident, &syn::Type) >, syn::Error > {
    let mut args = Vec::new();
    for arg in raw_args {
        match arg {
            syn::FnArg::Typed( syn::PatType { pat, ty, .. } ) => {
                let name = match &**pat {
                    syn::Pat::Ident( syn::PatIdent {
                        attrs,
                        by_ref: None,
                        mutability: None,
                        ident,
                        subpat: None
                    }) if attrs.is_empty() => {
                        ident
                    },
                    pat => return Err( syn::Error::new( pat.span(), "patterns like this are not supported" ) )
                };

                args.push( (name, &**ty) );
            },
            arg => return Err( syn::Error::new( arg.span(), "arguments like this are not supported" ) )
        }
    }

    Ok( args )
}

fn transform( mut input: syn::ItemFn ) -> Result< TokenStream, syn::Error > {
    let args = get_args( &input.sig.inputs )?;
    let return_ty = match input.sig.output {
        syn::ReturnType::Default => {
            syn::Type::Tuple( syn::TypeTuple {
                paren_token: Default::default(),
                elems: syn::punctuated::Punctuated::new()
            })
        },
        syn::ReturnType::Type( _, ty ) => (*ty).clone()
    };

    let flag_variable = syn::Ident::new( &format!( "_{}__flag_", input.sig.ident ), Span::call_site() );
    let return_variable = syn::Ident::new( &format!( "_{}__return_", input.sig.ident ), Span::call_site() );

    let mut global = Vec::new();
    global.push( quote! {
        #[cfg(test)]
        #[no_mangle]
        pub static mut #flag_variable: bool = false;

        #[cfg(test)]
        #[no_mangle]
        pub static mut #return_variable: *mut #return_ty = 0 as _;
    });

    let mut prelude = Vec::new();
    for (name, ty) in args {
        let variable = syn::Ident::new( &format!( "_{}__arg__{}_", input.sig.ident, name ), Span::call_site() );
        global.push( quote! {
            #[cfg(test)]
            #[no_mangle]
            pub static mut #variable: *mut #ty = 0 as _;
        });

        prelude.push( quote! {
            let #name: #ty = unsafe { core::ptr::read_volatile( #variable ) };
        });
    }

    let message = format!(
        concat!(
            "\x1B[2K", // Clear line.
            "\x1B[400D", // Move cursor to the left.
            "\x1B[A", // Move up.
            "\x1B[2K", // Clear line.
            "\x1B[B", // Move down.
            "\x1B[1;31m", // Highlight red.
            "error",
            "\x1B[0m", // Clear color.
            ": unreachable code can be reached in '",
            "\x1B[1;41m", // Highlight background red.
            "{}",
            "\x1B[0m", // Clear color.
        ),
        input.sig.ident
    );

    let original_block = input.block;
    let block = syn::parse2::< syn::Block >( quote! {{
        macro_rules! assume {
            ($expr:expr) => {
                if !$expr {
                    unsafe { core::hint::unreachable_unchecked(); }
                }
            }
        }

        macro_rules! static_unreachable {
            () => {{
                extern "C" {
                    #[link_name = #message]
                    fn trigger() -> !;
                }

                unsafe {
                    trigger();
                }
            }}
        }

        macro_rules! static_assert {
            ($expr:expr) => {
                if !$expr {
                    static_unreachable!();
                }
            }
        }

        if unsafe { core::ptr::read_volatile( &#flag_variable ) } {
            #(#prelude)*
            let _result_: #return_ty = (move || {
                #original_block
            })();
            unsafe { core::ptr::write_volatile( #return_variable, _result_ ) };
        }
    }})?;

    input.sig.inputs = Default::default();
    input.sig.output = syn::ReturnType::Default;
    input.block = Box::new( block );
    input.attrs.push( create_test_attr() );

    Ok( quote! {
        #(#global)*
        #input
    })
}


#[proc_macro_attribute]
pub fn static_test( attrs: proc_macro::TokenStream, input: proc_macro::TokenStream ) -> proc_macro::TokenStream {
    let _ = parse_macro_input!( attrs as Attributes );
    let input = parse_macro_input!( input as syn::ItemFn );

    let output = match transform( input ) {
        Ok( tokens ) => tokens,
        Err( error ) => error.to_compile_error()
    };

    proc_macro::TokenStream::from( output )
}
