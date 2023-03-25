// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(AntTx)]
pub fn derive_ant_tx(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_ant_tx(&ast)
}

fn impl_ant_tx(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl AntTxMessageType for #name {
            fn serialize_message(&self, buf: &mut [u8]) -> Result<usize, PackingError> {
                let len = PackedStructSlice::packed_bytes_size(Some(self))?;
                self.pack_to_slice(&mut buf[..len])?;
                Ok(len)
            }
            fn get_tx_msg_id(&self) -> TxMessageId {
                TxMessageId::#name
            }
        }
        impl From<#name> for TxMessage {
            fn from(msg: #name) -> TxMessage {
                TxMessage::#name(msg)
            }
        }
    };
    gen.into()
}

#[proc_macro_derive(DataPage)]
pub fn derive_data_page(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_data_page(&ast)
}

fn impl_data_page(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl #name {
            fn get_datapage_number(&self) -> u8 {
                self.data_page_number.into()
            }
        }
    };
    gen.into()
}
