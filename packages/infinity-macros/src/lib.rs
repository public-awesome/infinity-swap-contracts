use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, DataEnum, DeriveInput, Path};

fn infinity_interface_path(inside: &str) -> Path {
    let pkg = std::env::var("CARGO_PKG_NAME").unwrap();
    let base = if pkg == "infinity-shared" {
        "crate::interface"
    } else {
        "::infinity_shared::interface"
    };
    let path = format!("{base}::{inside}");
    let path: Path = syn::parse_str(&path).unwrap();
    path
}

// Merges the variants of two enums.
fn merge_variants(metadata: TokenStream, left: TokenStream, right: TokenStream) -> TokenStream {
    use syn::Data::Enum;

    let args = parse_macro_input!(metadata as AttributeArgs);
    if let Some(first_arg) = args.first() {
        return syn::Error::new_spanned(first_arg, "macro takes no arguments")
            .to_compile_error()
            .into();
    }

    let mut left: DeriveInput = parse_macro_input!(left);
    let right: DeriveInput = parse_macro_input!(right);

    if let (
        Enum(DataEnum { variants, .. }),
        Enum(DataEnum {
            variants: to_add, ..
        }),
    ) = (&mut left.data, right.data)
    {
        variants.extend(to_add.into_iter());

        quote! { #left }.into()
    } else {
        syn::Error::new(left.ident.span(), "variants may only be added for enums")
            .to_compile_error()
            .into()
    }
}

#[proc_macro_attribute]
pub fn infinity_module_query(metadata: TokenStream, input: TokenStream) -> TokenStream {
    let swap_response = infinity_interface_path("SwapResponse");
    let nft_order = infinity_interface_path("NftOrder");
    let swap_params = infinity_interface_path("SwapParams");
    let uint_128: Path = syn::parse_str("cosmwasm_std::Uint128").unwrap();

    merge_variants(
        metadata,
        input,
        quote! {
        enum Right {
            #[returns(#swap_response)]
            SimSwapNftsForTokens {
                sender: String,
                collection: String,
                nft_orders: Vec<#nft_order>,
                swap_params: #swap_params,
            },
            #[returns(#swap_response)]
            SimSwapTokensForAnyNfts {
                sender: String,
                collection: String,
                orders: Vec<#uint_128>,
                swap_params: #swap_params,
            },
        }
        }
        .into(),
    )
}

#[proc_macro_attribute]
pub fn infinity_module_execute(metadata: TokenStream, input: TokenStream) -> TokenStream {
    let nft_order = infinity_interface_path("NftOrder");
    let swap_params = infinity_interface_path("SwapParams");
    let uint_128: Path = syn::parse_str("cosmwasm_std::Uint128").unwrap();

    merge_variants(
        metadata,
        input,
        quote! {
        enum Right {
            SwapTokensForAnyNfts {
                collection: String,
                orders: Vec<#uint_128>,
                swap_params: #swap_params,
            },
            SwapNftsForTokens {
                collection: String,
                nft_orders: Vec<#nft_order>,
                swap_params: #swap_params,
            },
        }
        }
        .into(),
    )
}
