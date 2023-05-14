//!
//! This library provides the [`multicall!`] macro, which allows you to apply multiple operations
//! to one object without writing the name of the object again and again.
//!

#![no_std]

extern crate alloc;
extern crate proc_macro;
#[cfg(MULTICALL_DEBUG)]
extern crate std;

#[cfg(MULTICALL_DEBUG)]
use std::println;

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
    #[cfg(MULTICALL_DEBUG)]
    println!("creating new multicall block...");
    let mut dat = if is_recursed {
        #[cfg(MULTICALL_DEBUG)]
        println!("inserting multicall item because this is a recursed block.");
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
    #[cfg(MULTICALL_DEBUG)]
    println!("initialized. reading item...");
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
    #[cfg(MULTICALL_DEBUG)]
    println!("item read. writing initial let statement.");
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
    #[cfg(MULTICALL_DEBUG)]
    println!("done. starting state machine and processing statements.");
    #[derive(Default, PartialEq, Eq)]
    enum State {
        #[default]
        InsertNew,
        Set,
        Inserted,
    }
    #[derive(Default)]
    struct AccumState {
        words: Vec<TokenTree>,
        state: State,
    }
    ts.extend(
        iter.fold(AccumState::default(), |mut accum, x| {
            let o = x.to_string();
            // Sub-calls
            if let Some(x) = match x {
                TokenTree::Group(ref x) if accum.state == State::InsertNew => Some(x),
                _ => None,
            } {
                #[cfg(MULTICALL_DEBUG)]
                println!("found group, making sub-call:");
                accum
                    .words
                    .extend(multicall_internal(x.stream(), true, is_mut).into_iter());
                accum.state = State::Inserted;
                #[cfg(MULTICALL_DEBUG)]
                println!("sub-call inserted.");
            // End of call
            } else if o == ";" {
                #[cfg(MULTICALL_DEBUG)]
                println!("found semicolon. resetting.");
                accum.state = State::InsertNew;
                accum
                    .words
                    .push(TokenTree::Punct(Punct::new(';', Spacing::Alone)));
            // Call content
            } else {
                #[cfg(MULTICALL_DEBUG)]
                println!("found statement. parsing...");
                if accum.state == State::InsertNew {
                    #[cfg(MULTICALL_DEBUG)]
                    println!("detecting statement type...");
                    if o == "set" {
                        #[cfg(MULTICALL_DEBUG)]
                        println!("statement is 'set'.");
                        accum.state = State::Set;
                        return accum; // dont insert
                    } else if o == "exec" {
                        #[cfg(MULTICALL_DEBUG)]
                        println!("statement is 'exec'. marking for full replay.");
                        accum.state = State::Inserted;
                        return accum; // dont insert
                    }
                    #[cfg(MULTICALL_DEBUG)]
                    println!("inserting item.");
                    accum.words.push(TokenTree::Ident(Ident::new(
                        "__multicall_item__",
                        Span::call_site(),
                    )));
                    accum
                        .words
                        .push(TokenTree::Punct(Punct::new('.', Spacing::Alone)));
                    accum.state = State::Inserted;
                    #[cfg(MULTICALL_DEBUG)]
                    println!("done. replaying rest.");
                }
                if accum.state == State::Set {
                    if o == "=" {
                        #[cfg(MULTICALL_DEBUG)]
                        println!("replaying '='.");
                        accum.state = State::InsertNew;
                    }
                }
                #[cfg(MULTICALL_DEBUG)]
                println!("replaying '{x}'");
                accum
                    .words
                    .push(recursive_replace(x, "#", "__multicall_item__"));
            }
            accum
        })
        .words,
    );
    #[cfg(MULTICALL_DEBUG)]
    println!("multicall block done.");
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
