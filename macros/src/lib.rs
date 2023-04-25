use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Table, attributes(column))]
pub fn derive_column(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;

    if let Data::Struct(data) = input.data {
        if let Fields::Named(fields) = data.fields {
            let idents = fields.named.iter().filter_map(|field| {
                for attr in field.attrs.iter() {
                    if attr.path.segments.len() == 1
                        && attr.path.segments[0].ident.to_string() == "column"
                    {
                        if let Some(ref i) = field.ident {
                            return Some(i.to_string());
                        }

                        return None;
                    }
                }

                None
            });

            let columns = quote! { #(#idents), * };

            quote! {
                impl ColumnsTrait for #ident {
                    fn columns() -> &'static[&'static str] {
                        return &[#columns]
                    }
                }
            }
            .into()
        } else {
            panic!("no named fields")
        }
    } else {
        panic!("not a struct")
    }
}
