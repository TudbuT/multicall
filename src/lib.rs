//!
//! This library provides the [`multicall!`] macro, which allows you to apply multiple operations
//! to one object without writing the name of the object again and again.
//!

#![no_std]

extern crate alloc;
extern crate proc_macro;

use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

/// Macro to execute multiple operations on one object in a short form.
///
/// Syntax:
/// ```ignore
/// let mut test_variable = 1;
/// multicall! {
///     expr:
///     operation;
///     set test_variable = operation;
///     exec normal_operation(#);
///     operation;
///     ...
///     {
///         subexpr:
///         operation;
///         set test_variable += operation;
///         exec normal_operation(#);
///         operation;
///         ...
///     }; // this semicolon is mandatory.
/// }
/// ```
///
/// Evaluates to:
/// ```ignore
/// let mut test_variable = 1;
/// {
///     let __multicall_item__ = expr;
///     __multicall_item__.operation;
///     test_variable = __multicall_item__.operation;
///     normal_operation(__multicall_item__);
///     __multicall_item__.operation;
///     ...
///     {
///         let __multicall_item__ = __multicall_item__.subexpr;
///         __multicall_item__.operation;
///         test_variable += __multicall_item__.operation;
///         normal_operation(__multicall_item__);
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
///     let b_plus_five;
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
///         set b_plus_five = b + 5;
///         exec println!("{}, {}", #.a, #.b);
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
        if let TokenTree::Punct(ref x) = item {
            if x.as_char() == ':' && x.spacing() == Spacing::Alone {
                break;
            }
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
        iter.fold((true, Vec::new(), false, false), |mut accum, x| {
            let o = x.to_string();
            // Sub-calls
            if let Some(x) = match x {
                TokenTree::Group(ref x) if accum.0 => Some(x),
                _ => None,
            } {
                accum
                    .1
                    .extend(multicall_internal(x.stream(), true, is_mut).into_iter());
                accum.0 = false;
            // End of call
            } else if o == ";" {
                accum.0 = true;
                accum.2 = false;
                accum.3 = false;
                accum
                    .1
                    .push(TokenTree::Punct(Punct::new(';', Spacing::Alone)));
            // Call content
            } else {
                if accum.0 {
                    if o == "set" {
                        accum.2 = true;
                        return accum; // dont include
                    } else if accum.2 {
                        if o == "=" {
                            accum.2 = false;
                        }
                    } else if o == "exec" {
                        accum.3 = true;
                        accum.0 = false;
                        return accum; // dont include
                    } else if !accum.3 {
                        accum.1.push(TokenTree::Ident(Ident::new(
                            "__multicall_item__",
                            Span::call_site(),
                        )));
                        accum
                            .1
                            .push(TokenTree::Punct(Punct::new('.', Spacing::Alone)));
                        accum.0 = false;
                    }
                }
                accum
                    .1
                    .push(recursive_replace(x, "#", "__multicall_item__"));
            }
            accum
        })
        .1,
    );
    TokenStream::from(TokenTree::Group(Group::new(Delimiter::Brace, ts)))
}

fn recursive_replace(token: TokenTree, from: &str, to: &str) -> TokenTree {
    match token {
        TokenTree::Group(x) => TokenTree::Group({
            let mut g = Group::new(
                x.delimiter(),
                TokenStream::from_iter(
                    x.stream()
                        .into_iter()
                        .map(|x| recursive_replace(x, from, to)),
                ),
            );
            g.set_span(x.span());
            g
        }),
        TokenTree::Ident(x) => TokenTree::Ident(Ident::new(
            x.to_string().replace(from, to).as_str(),
            x.span(),
        )),
        TokenTree::Punct(x) if x.as_char() == from.chars().next().unwrap() && from.len() == 1 => {
            TokenTree::Ident(Ident::new(to, x.span()))
        }
        x => x,
    }
}
