use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, AttributeArgs, FnArg, ItemFn, Lit, Meta, NestedMeta, Pat, Type};

pub fn tool_function_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input_fn = parse_macro_input!(input as ItemFn);

    // Parse the arguments
    let mut tool_name = None;
    let mut tool_description = None;

    for arg in args {
        match arg {
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("name") => {
                if let Lit::Str(lit_str) = nv.lit {
                    tool_name = Some(lit_str.value());
                }
            }
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("description") => {
                if let Lit::Str(lit_str) = nv.lit {
                    tool_description = Some(lit_str.value());
                }
            }
            _ => {
                return syn::Error::new_spanned(arg, "Expected name = \"...\" or description = \"...\"")
                    .to_compile_error()
                    .into();
            }
        }
    }

    let tool_name = tool_name.unwrap_or_else(|| input_fn.sig.ident.to_string());
    let tool_description = tool_description.unwrap_or_else(|| format!("Tool function: {}", tool_name));

    // Validate the function signature
    if input_fn.sig.asyncness.is_none() {
        return syn::Error::new_spanned(&input_fn.sig, "Tool functions must be async")
            .to_compile_error()
            .into();
    }

    if input_fn.sig.inputs.len() != 1 {
        return syn::Error::new_spanned(
            &input_fn.sig.inputs,
            "Tool functions must take exactly one parameter",
        )
        .to_compile_error()
        .into();
    }

    // Extract the parameter type
    let param_type = match &input_fn.sig.inputs[0] {
        FnArg::Typed(pat_type) => &pat_type.ty,
        _ => {
            return syn::Error::new_spanned(
                &input_fn.sig.inputs[0],
                "Tool function parameter must be a typed parameter",
            )
            .to_compile_error()
            .into();
        }
    };

    // Extract the return type
    let return_type = match &input_fn.sig.output {
        syn::ReturnType::Type(_, ty) => ty,
        _ => {
            return syn::Error::new_spanned(
                &input_fn.sig,
                "Tool functions must have an explicit return type",
            )
            .to_compile_error()
            .into();
        }
    };

    // Parse Result<T, E> from return type
    let (output_type, error_type) = match extract_result_types(return_type) {
        Some(types) => types,
        None => {
            return syn::Error::new_spanned(
                return_type,
                "Tool functions must return Result<T, E>",
            )
            .to_compile_error()
            .into();
        }
    };

    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let struct_name = syn::Ident::new(&format!("{}Tool", fn_name), fn_name.span());
    let tool_fn_name = syn::Ident::new(&format!("{}_tool", fn_name), fn_name.span());

    let expanded = quote! {
        #input_fn

        #[derive(Clone)]
        #fn_vis struct #struct_name;

        impl genai_tools::ToolFunction for #struct_name {
            type Params = #param_type;
            type Output = #output_type;
            type Error = #error_type;

            fn name(&self) -> &'static str {
                #tool_name
            }

            fn description(&self) -> &'static str {
                #tool_description
            }

            fn call(&self, params: Self::Params) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Output, Self::Error>> + Send + '_>> {
                Box::pin(async move {
                    #fn_name(params).await
                })
            }
        }

        // Create a function that returns the tool instance for registration
        #fn_vis fn #tool_fn_name() -> #struct_name {
            #struct_name
        }
    };

    TokenStream::from(expanded)
}

fn extract_result_types(ty: &Type) -> Option<(&Type, &Type)> {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Result" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if args.args.len() == 2 {
                        if let (syn::GenericArgument::Type(ok_type), syn::GenericArgument::Type(err_type)) =
                            (&args.args[0], &args.args[1])
                        {
                            return Some((ok_type, err_type));
                        }
                    }
                }
            }
        }
    }
    None
} 