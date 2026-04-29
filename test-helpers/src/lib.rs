use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn api_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_output = &input_fn.sig.output;
    let fn_body = &input_fn.block;

    let expanded = quote! {
        #[tokio::test]
        async fn #fn_name(#fn_inputs) #fn_output {
            let mut app = TestApp::new().await;
            #fn_body
            app.cleanup().await;
        }
    };

    expanded.into()
}
