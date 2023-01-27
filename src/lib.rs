//!
//! This library provides the [`multicall!`] macro, which allows you to apply multiple operations
//! to one object without writing the name of the object again and again.
//!

#![no_std]

extern crate proc_macro;
extern crate alloc;
use alloc::string::ToString;
use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};
use alloc::vec::Vec;
use alloc::vec;

/// Macro to execute multiple operations on one object in a short form.
///
/// Syntax:
/// ```
/// multicall! {
///     expr:
///     operation;
///     operation;
///     operation;
///     ...
///     {
///         subexpr:
///         operation;
///         operation;
///         operation;
///         ...
///         ...
///     }; // this semicolon is mandatory.
/// }
/// ```
/// 
/// Evaluates to:
/// ```
/// {
///     let __multicall_item__ = expr;
///     __multicall_item__.operation;
///     __multicall_item__.operation;
///     __multicall_item__.operation;
///     ...
///     {
///         let __multicall_item__ = __multicall_item__.subexpr;
///         __multicall_item__.operation;
///         __multicall_item__.operation;
///         __multicall_item__.operation;
///         ...
///     };
/// }
/// ```
///
/// Example:
///    
/// ```
/// use multicall::multicall;
/// use std::ops::AddAssign;
/// #[derive(Debug)]
/// struct Test { a: u32, b: i32 }
///
/// fn main() {
///     let mut test = Test { a: 0, b: 0 };
///     multicall! {
///         &mut test:
///         a = 5;
///         b = 6;
///         {
///             b:
///             add_assign(500);
///         };
///         {
///             a:
///             add_assign(58);
///         };
///         a.add_assign(100 - 58);
///     }
///     println!("{test:?}");
/// }
/// ```
///
#[proc_macro]
pub fn multicall(input: TokenStream) -> TokenStream {
    multicall_internal(input, false, false)
}

fn multicall_internal(input: TokenStream, is_recursed: bool, mut is_mut: bool) -> TokenStream {
    let mut iter = input.into_iter();
    let mut dat = if is_recursed {
        let mut v = vec![
            TokenTree::Punct(Punct::new('&', Spacing::Alone)),
            TokenTree::Ident(Ident::new("mut", Span::call_site())),
            TokenTree::Ident(Ident::new("__multicall_item__", Span::call_site())),
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
        ];
        if !is_mut {
            v.remove(1);
        }
        v
    } else {
        Vec::new()
    };
    while let Some(item) = iter.next() {
        if item.to_string() == ":" {
            break;
        }
        if item.to_string() == "mut" && dat.len() == 1 {
            is_mut = true;
        }
        dat.push(item)
    }
    let mut ts = TokenStream::new();
    ts.extend(
        vec![
            TokenTree::Ident(Ident::new("let", Span::call_site())),
            TokenTree::Ident(Ident::new("__multicall_item__", Span::call_site())),
            TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        ]
        .into_iter(),
    );
    ts.extend(dat.clone().into_iter());
    ts.extend(vec![TokenTree::Punct(Punct::new(';', Spacing::Alone))].into_iter());
    ts.extend(
        iter.fold((true, Vec::new()), |mut accum, x| {
            let o = x.to_string();
            if let Some(x) = match x {
                TokenTree::Group(ref x) if accum.0 => Some(x),
                _ => None,
            } {
                accum
                    .1
                    .extend(multicall_internal(x.stream(), true, is_mut).into_iter());
                accum.0 = false;
            } else if o == ";" {
                accum.0 = true;
                accum
                    .1
                    .push(TokenTree::Punct(Punct::new(';', Spacing::Alone)));
            } else {
                if accum.0 {
                    accum.1.push(TokenTree::Ident(Ident::new(
                        "__multicall_item__",
                        Span::call_site(),
                    )));
                    accum
                        .1
                        .push(TokenTree::Punct(Punct::new('.', Spacing::Alone)));
                    accum.0 = false;
                }
                accum.1.push(x);
            }
            accum
        })
        .1,
    );
    TokenStream::from(TokenTree::Group(Group::new(Delimiter::Brace, ts)))
}


