use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Type, PathArguments, GenericArgument};

// Helper function to convert snake_case to UpperCamelCase
fn to_upper_camel_case(input: &str) -> String {
    input
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars.as_str().to_lowercase().chars()).collect(),
            }
        })
        .collect()
}

/// The main macro for defining tool functions.
#[proc_macro_attribute]
pub fn tool_function(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    // Parse the arguments using syn 2.0 style
    let mut tool_name = None;
    let mut tool_description = None;

    if !args.is_empty() {
        let args_str = args.to_string();
        
        // Simple parsing for name = "..." and description = "..."
        for part in args_str.split(',') {
            let part = part.trim();
            if let Some(name_value) = part.strip_prefix("name") {
                if let Some(value) = extract_string_literal(name_value) {
                    tool_name = Some(value);
                }
            } else if let Some(desc_value) = part.strip_prefix("description") {
                if let Some(value) = extract_string_literal(desc_value) {
                    tool_description = Some(value);
                }
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
    
    // Convert function name to UpperCamelCase and append "Tool"
    let struct_name_str = format!("{}Tool", to_upper_camel_case(&fn_name.to_string()));
    let struct_name = syn::Ident::new(&struct_name_str, fn_name.span());
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

// Helper function to extract string literals from attribute arguments
fn extract_string_literal(input: &str) -> Option<String> {
    let input = input.trim();
    if let Some(eq_part) = input.strip_prefix("=") {
        let eq_part = eq_part.trim();
        if eq_part.starts_with('"') && eq_part.ends_with('"') && eq_part.len() >= 2 {
            return Some(eq_part[1..eq_part.len()-1].to_string());
        }
    }
    None
}

fn extract_result_types(ty: &Type) -> Option<(&Type, &Type)> {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Result" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if args.args.len() == 2 {
                        if let (GenericArgument::Type(ok_type), GenericArgument::Type(err_type)) =
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_upper_camel_case() {
        assert_eq!(to_upper_camel_case("snake_case"), "SnakeCase");
        assert_eq!(to_upper_camel_case("get_weather"), "GetWeather");
        assert_eq!(to_upper_camel_case("simple"), "Simple");
        assert_eq!(to_upper_camel_case("very_long_function_name"), "VeryLongFunctionName");
        assert_eq!(to_upper_camel_case("already_Capital"), "AlreadyCapital");
        assert_eq!(to_upper_camel_case(""), "");
        assert_eq!(to_upper_camel_case("single"), "Single");
    }
}