// rt_attrs/src/lib.rs
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, ImplItemFn, Item, ItemFn, LitStr, Stmt,
    Token, TraitItemFn,
};
#[proc_macro_attribute]
pub fn realtime(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Explicitly ignore unused attribute arguments (allow empty args)
    let _ = attr;
    // let _ = parse_macro_input!(attr as syn::parse::Nothing); // 允许空参数（不接收任何参数）
    // let f = parse_macro_input!(item as ItemFn);
    // let vis = &f.vis; let sig = &f.sig; let block = &f.block; let attrs = &f.attrs;
    // quote! {
    //     #(#attrs)*
    //     #[doc = "rt:realtime"]
    //     #vis #sig #block
    // }.into()
    // 1) Methods in traits
    if let Ok(mut m) = syn::parse::<TraitItemFn>(item.clone()) {
        m.attrs.push(syn::parse_quote!(#[doc = "rt:realtime"]));
        return quote!(#m).into();
    }
    // 2) Methods in impl blocks
    if let Ok(mut m) = syn::parse::<ImplItemFn>(item.clone()) {
        m.attrs.push(syn::parse_quote!(#[doc = "rt:realtime"]));
        return quote!(#m).into();
    }
    // 3) Free functions
    if let Ok(mut f) = syn::parse::<ItemFn>(item.clone()) {
        f.attrs.push(syn::parse_quote!(#[doc = "rt:realtime"]));
        return quote!(#f).into();
    }
    // Return as-is elsewhere (or error out)
    item
}

#[proc_macro_attribute]
pub fn non_realtime(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Explicitly ignore unused attribute arguments (allow empty args)
    let _ = attr;
    // let _ = parse_macro_input!(attr as syn::parse::Nothing); // 允许空参数（不接收任何参数）
    // let f = parse_macro_input!(item as ItemFn);
    // let vis = &f.vis; let sig = &f.sig; let block = &f.block; let attrs = &f.attrs;
    // quote! {
    //     #(#attrs)*
    //     #[doc = "rt:realtime"]
    //     #vis #sig #block
    // }.into()
    // 1) Methods in traits
    if let Ok(mut m) = syn::parse::<TraitItemFn>(item.clone()) {
        m.attrs.push(syn::parse_quote!(#[doc = "rt:non_realtime"]));
        return quote!(#m).into();
    }
    // 2) Methods in impl blocks
    if let Ok(mut m) = syn::parse::<ImplItemFn>(item.clone()) {
        m.attrs.push(syn::parse_quote!(#[doc = "rt:non_realtime"]));
        return quote!(#m).into();
    }
    // 3) Free functions
    if let Ok(mut f) = syn::parse::<ItemFn>(item.clone()) {
        f.attrs.push(syn::parse_quote!(#[doc = "rt:non_realtime"]));
        return quote!(#f).into();
    }
    // Return as-is elsewhere (or error out)
    item
}
/// Define the #[rt_call_info("function_name"|"closure", "realtime"|"nonrealtime")] attribute macro
///
/// This macro is a pure marker and does not generate runtime code. It is used to:
/// - Validate parameter format at compile time (two string literals).
/// - Preserve the marker attribute in code for static analysis by Dylint and other linters.
#[proc_macro_attribute]
pub fn rt_call_info(
    // `args`: two strings. The first is the function name pointed to by the function pointer ("closure" for closures), the second is realtime property ("realtime" or "nonrealtime").
    args: TokenStream,
    // `item`: can be attached to a statement (like let) or an item (like fn).
    item: TokenStream,
) -> TokenStream {
    // Step 1: Parse and validate the attribute arguments.
    // Ensure the user provides exactly two string literals.
    let list = parse_macro_input!(args with Punctuated::<LitStr, Token![,]>::parse_terminated);

    if list.len() != 2 {
        panic!(
            "rt_call_info expects exactly two string literals, e.g. #[rt_call_info(\"foo\"|\"closure\", \"realtime\"|\"nonrealtime\")]"
        );
    }

    let first_val = list[0].value();
    let second_val = list[1].value();

    // Print a note at compile time to indicate the macro was invoked
    eprintln!(
        "[rt_attrs] rt_call_info attribute invoked with: \"{}\", \"{}\"",
        first_val, second_val
    );

    // Using the raw string literals, construct a lazy (non-procedural) marker attribute
    // that is preserved after macro expansion, for Dylint to detect at the HIR stage.
    let first_lit = &list[0];
    let second_lit = &list[1];
    // Use a doc marker to avoid requiring unstable register_tool features downstream
    let marker_attr = parse_quote!(#[doc = concat!("rt:call-info:", #first_lit, ":", #second_lit)]);

    // Convert input to proc_macro2 for flexible parsing as Item or Stmt
    let input_ts2: proc_macro2::TokenStream = item.clone().into();

    // 1) First, try parsing as a statement (supporting let declarations)
    if let Ok(stmt) = syn::parse2::<Stmt>(input_ts2.clone()) {
        if let Stmt::Local(mut local) = stmt {
            // Attach marker attribute to the let statement
            local.attrs.push(marker_attr);
            let new_stmt = Stmt::Local(local);
            return TokenStream::from(new_stmt.into_token_stream());
        } else {
            // Not a Local statement, return as-is
            return item;
        }
    }

    // 2) Try parsing as an item (e.g., function)
    if let Ok(mut node) = syn::parse2::<Item>(input_ts2) {
        match &mut node {
            Item::Fn(item_fn) => {
                item_fn.attrs.push(marker_attr);
            }
            // Other Item types can be supported by adding attrs push as needed
            _ => {}
        }
        return TokenStream::from(node.into_token_stream());
    }

    // 3) If parsing fails, preserve the original input (safe fallback)
    item
}
