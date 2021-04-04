use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn rmp_parallel_for(args: TokenStream, func: TokenStream) -> TokenStream {
    func
}

