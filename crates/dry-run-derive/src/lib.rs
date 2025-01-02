extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parser, parse_macro_input, Data, DeriveInput, Fields, ItemStruct};

#[proc_macro_attribute]
pub fn force(_args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree (expecting a struct).
    let mut item_struct = parse_macro_input!(input as ItemStruct);

    // Add the new field to the struct if it's a named field struct.
    if let Fields::Named(ref mut fields) = item_struct.fields {
        fields.named.push(
            syn::Field::parse_named
                .parse2(quote! {
                    /// Indicates whether this operation is a dry-run.
                    #[clap(short, long, default_value_t = false)]
                    pub force: bool
                })
                .expect("Failed to parse the field 'force'."),
        );
    } else {
        // Generate a compile error for unsupported struct types.
        return quote! {
            compile_error!("The `force` macro can only be applied to structs with named fields.");
        }
        .into();
    }

    // Return the modified struct.
    quote! {
        #item_struct
    }
    .into()
}

#[proc_macro_attribute]
pub fn dry_run(_args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree (expecting a struct).
    let mut item_struct = parse_macro_input!(input as ItemStruct);

    // Add the new field to the struct if it's a named field struct.
    if let Fields::Named(ref mut fields) = item_struct.fields {
        fields.named.push(
            syn::Field::parse_named
                .parse2(quote! {
                    /// Indicates whether this operation is a dry-run.
                    #[clap(short = 'n', long, default_value_t = false)]
                    pub dry_run: bool
                })
                .expect("Failed to parse the new field."),
        );
    } else {
        // Generate a compile error for unsupported struct types.
        return quote! {
            compile_error!("The `dry_run` macro can only be applied to structs with named fields.");
        }
        .into();
    }

    // Return the modified struct.
    quote! {
        #item_struct
    }
    .into()
}

/// A derive macro to append a `dry_run` field to structs, used for testing purposes.
///
/// This macro ensures that the generated struct contains a `dry_run` field, which is a boolean flag
/// indicating whether operations should be executed as actual or simulated (dry-run).
///
/// ## Struct Fields
///
/// - `dry_run`
///
///     A boolean flag. When set to `true`, the struct operates in a "dry-run" mode, avoiding any
///     permanent changes or side effects.
///
/// ## Limitations
///
/// This macro can only be used on structs with named fields.
/// Attempting to use it on tuple structs or unit structs will result in a compilation error.
#[proc_macro_derive(DryRun)]
pub fn derive_dry_run(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree.
    let ast = parse_macro_input!(input as DeriveInput);

    // Generate the implementation of the DryRun derive macro.
    impl_derive_dry_run(&ast)
}

fn impl_derive_dry_run(ast: &DeriveInput) -> TokenStream {
    let struct_name = &ast.ident;
    let expanded = if let Data::Struct(data_struct) = &ast.data {
        if let Fields::Named(fields) = &data_struct.fields {
            let field_defs = quote! {
                #[clap::arg(short = 'n', long, verbatim_doc_comment)]
                #[clap::arg(default_value_t = true)]
                dry_run: bool,
            };

            // Retain the existing fields and append the common ones.
            let original_fields = fields.named.iter();
            quote! {
                pub struct #struct_name {
                    #(#original_fields,)*
                    #field_defs
                }
            }
        } else {
            // Error message if used on structs without named fields.
            quote! {
                compile_error!("ModelProps can only be derived for structs with named fields.");
            }
        }
    } else {
        // Error message if used on non-struct data types.
        quote! {
            compile_error!("ModelProps can only be used with structs.");
        }
    };
    // Return the generated TokenStream.
    expanded.into()
}
