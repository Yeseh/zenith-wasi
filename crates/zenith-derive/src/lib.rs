use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, AttributeArgs, ItemFn, Lit};

#[proc_macro_attribute]
pub fn main(args: TokenStream, item: TokenStream) -> TokenStream {
    attr_run_impl(args, item)
}

fn attr_run_impl(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let item = parse_macro_input!(item as ItemFn);

    let fn_name_ident = item.sig.ident.clone();
    let input_arg = item.sig.inputs.first().unwrap();

    let mut arg_lits: Vec<syn::Lit> = Vec::new();
    let mut deserializer: proc_macro2::TokenStream = proc_macro2::TokenStream::default();
    let mut input_type: proc_macro2::TokenStream = proc_macro2::TokenStream::default();

    match input_arg {
        syn::FnArg::Receiver(_) => todo!(),
        syn::FnArg::Typed(typed) => input_type = typed.ty.to_token_stream(),
    };

    // TODO: Make this more robust :)
    for arg in args {
        match arg {
            syn::NestedMeta::Meta(meta) => match meta {
                syn::Meta::Path(path) => deserializer = path.segments.into_token_stream(),
                syn::Meta::List(_) => todo!(),
                syn::Meta::NameValue(_) => todo!(),
            },
            syn::NestedMeta::Lit(lit) => arg_lits.push(lit),
        }
    }

    let input_size_str = arg_lits
        .iter()
        .nth(0)
        .unwrap()
        .to_token_stream()
        .to_string();

    let output_size_str = arg_lits
        .iter()
        .nth(1)
        .unwrap()
        .to_token_stream()
        .clone()
        .to_string();

    let input_size = usize::from_str_radix(&input_size_str, 10).unwrap();
    let output_size = usize::from_str_radix(&output_size_str, 10).unwrap();

    quote! {
        use zenith::prelude::{ZenDeserializer};

        const INPUT_BUFFER_SIZE: usize = #input_size;
        const OUTPUT_BUFFER_SIZE: usize = #output_size;

        static mut INPUT_BUFFER: [u8; INPUT_BUFFER_SIZE] = [0; INPUT_BUFFER_SIZE];
        static mut OUTPUT_BUFFER: [u8; OUTPUT_BUFFER_SIZE] = [0; OUTPUT_BUFFER_SIZE];

        #[no_mangle]
        pub fn get_input_buffer_size() -> i32 {
            INPUT_BUFFER_SIZE as i32
        }

        #[no_mangle]
        pub fn get_output_buffer_size() -> i32 {
            OUTPUT_BUFFER_SIZE as i32
        }

        #[no_mangle]
        pub fn get_input_buffer_pointer() -> i32 {
            let pointer: *const u8;
            unsafe {
                pointer = INPUT_BUFFER.as_ptr();
            }
            pointer as i32
        }

        #[no_mangle]
        pub fn get_output_buffer_pointer() -> i32 {
            let pointer: *const u8;
            unsafe {
                pointer = OUTPUT_BUFFER.as_ptr();
            }
            pointer as i32
        }

        #[no_mangle]
        pub fn write_input(idx: i32, val: i32) -> i32 {
            unsafe {
                INPUT_BUFFER[idx as usize] = val as u8;
            }

            val
        }

        #[no_mangle]
        pub fn read_output(idx: i32) -> i32 {
            unsafe { OUTPUT_BUFFER[idx as usize] as i32 }
        }

        fn write_output(data: &[u8]) -> i32 {
            let mut written = 0;
            for (i, byte) in data.iter().enumerate() {
                let val = *byte;
                unsafe { OUTPUT_BUFFER[i] = val }
                written += 1;
            }

            written
        }

        #item

        #[no_mangle]
        pub fn __zen_run(input_offset: i32, input_size: i32) -> i32 {
            let slice: &[u8];
            let start_ptr = get_input_buffer_pointer() as *const u8;

            unsafe {
                let start = start_ptr.add(input_offset as usize);
                slice = slice::from_raw_parts(start, input_size as usize);
            }

            // TODO: Better error handling here for the user
            let input_data: #input_type = bincode::deserialize(slice).unwrap();
            let output_data = #fn_name_ident(input_data);
            let output = bincode::serialize(&output_data).unwrap();
            let written = write_output(output.as_slice());

            written
        }
    }
    .into()
}
