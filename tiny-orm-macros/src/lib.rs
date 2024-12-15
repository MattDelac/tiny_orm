use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use types::Operation;

mod attr;
mod database;
mod quotes;
mod types;

#[proc_macro_derive(Table, attributes(tiny_orm))]
pub fn derive_tiny_orm(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let attr = attr::Attr::parse(input);

    let expanded = generate_impl(&attr);

    #[cfg(test)]
    println!("Generated code:\n{}", expanded);

    TokenStream::from(expanded)
}

fn generate_impl(attr: &attr::Attr) -> proc_macro2::TokenStream {
    let struct_name = attr.parsed_struct.name.clone();

    let get_impl = if attr.operations.contains(&Operation::Get) {
        quotes::get_by_id_fn(attr)
    } else {
        quote! {}
    };

    let list_impl = if attr.operations.contains(&Operation::List) {
        quotes::list_all_fn(attr)
    } else {
        quote! {}
    };

    let create_impl = if attr.operations.contains(&Operation::Create) {
        quotes::create_fn(attr)
    } else {
        quote! {}
    };

    let update_impl = if attr.operations.contains(&Operation::Update) {
        quotes::update_fn(attr)
    } else {
        quote! {}
    };

    let delete_impl = if attr.operations.contains(&Operation::Delete) {
        quotes::delete_fn(attr)
    } else {
        quote! {}
    };

    quote! {
        impl #struct_name {
            #get_impl
            #list_impl
            #create_impl
            #update_impl
            #delete_impl
        }
    }
}
