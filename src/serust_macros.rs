// SELinux-style Rust (SErust) Macros and Clippy Lints
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, AttributeArgs, Lit, Meta, NestedMeta};

/// SErust domain declaration macro
#[proc_macro_attribute]
pub fn serust_domain(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input_fn = parse_macro_input!(input as ItemFn);

    let mut domain_level = 0u8;
    let mut allowed_orbits = Vec::new();
    let mut capabilities = Vec::new();

    // Parse domain attributes
    for arg in args {
        match arg {
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("level") => {
                if let Lit::Int(lit_int) = nv.lit {
                    domain_level = lit_int.base10_parse().unwrap_or(0);
                }
            }
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("orbits") => {
                if let Lit::Str(lit_str) = nv.lit {
                    allowed_orbits = lit_str.value().split(',').map(|s| s.trim().to_string()).collect();
                }
            }
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("capabilities") => {
                if let Lit::Str(lit_str) = nv.lit {
                    capabilities = lit_str.value().split(',').map(|s| s.trim().to_string()).collect();
                }
            }
            _ => {}
        }
    }

    let fn_name = &input_fn.sig.ident;
    let fn_block = &input_fn.block;
    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;

    TokenStream::from(quote! {
        #[serust_domain_info(level = #domain_level, orbits = #(#allowed_orbits),*, capabilities = #(#capabilities),*)]
        #fn_vis #fn_sig {
            // Compile-time domain verification
            const _DOMAIN_CHECK: () = {
                if #domain_level > 4 {
                    panic!("Invalid domain level");
                }
            };

            // Runtime domain enforcement
            serust_runtime::enforce_domain_access(#domain_level, &[#(#allowed_orbits),*], &[#(#capabilities),*]);

            #fn_block
        }
    })
}

/// Orbit classification macro
#[proc_macro_attribute]
pub fn orbit(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input_fn = parse_macro_input!(input as ItemFn);

    let mut orbit_class = "trivial".to_string();
    let mut complexity = "O(1)".to_string();

    for arg in args {
        match arg {
            NestedMeta::Meta(Meta::Path(path)) if path.is_ident("trivial") => {
                orbit_class = "trivial".to_string();
                complexity = "O(1)".to_string();
            }
            NestedMeta::Meta(Meta::Path(path)) if path.is_ident("cyclic") => {
                orbit_class = "cyclic".to_string();
                complexity = "O(n)".to_string();
            }
            NestedMeta::Meta(Meta::Path(path)) if path.is_ident("symmetric") => {
                orbit_class = "symmetric".to_string();
                complexity = "O(n!)".to_string();
            }
            NestedMeta::Meta(Meta::Path(path)) if path.is_ident("alternating") => {
                orbit_class = "alternating".to_string();
                complexity = "O(2^n)".to_string();
            }
            NestedMeta::Meta(Meta::Path(path)) if path.is_ident("sporadic") => {
                orbit_class = "sporadic".to_string();
                complexity = "irregular".to_string();
            }
            NestedMeta::Meta(Meta::Path(path)) if path.is_ident("monster") => {
                orbit_class = "monster".to_string();
                complexity = "unrestricted".to_string();
            }
            _ => {}
        }
    }

    let fn_name = &input_fn.sig.ident;
    let fn_block = &input_fn.block;
    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;

    TokenStream::from(quote! {
        #[orbit_info(class = #orbit_class, complexity = #complexity)]
        #fn_vis #fn_sig {
            // Compile-time orbit verification
            serust_runtime::verify_orbit_access(#orbit_class);

            #fn_block
        }
    })
}

/// Security capability requirement macro
#[proc_macro_attribute]
pub fn requires(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input_fn = parse_macro_input!(input as ItemFn);

    let mut required_caps = Vec::new();

    for arg in args {
        if let NestedMeta::Meta(Meta::Path(path)) = arg {
            if let Some(ident) = path.get_ident() {
                required_caps.push(ident.to_string());
            }
        }
    }

    let fn_name = &input_fn.sig.ident;
    let fn_block = &input_fn.block;
    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;

    TokenStream::from(quote! {
        #[capability_check(requires = #(#required_caps),*)]
        #fn_vis #fn_sig {
            // Runtime capability check
            serust_runtime::check_capabilities(&[#(#required_caps),*])?;

            #fn_block
        }
    })
}

/// Syscall allowlist macro
#[proc_macro_attribute]
pub fn allow_syscalls(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input_fn = parse_macro_input!(input as ItemFn);

    let mut allowed_syscalls = Vec::new();

    for arg in args {
        if let NestedMeta::Lit(Lit::Str(lit_str)) = arg {
            allowed_syscalls.push(lit_str.value());
        }
    }

    let fn_name = &input_fn.sig.ident;
    let fn_block = &input_fn.block;
    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;

    TokenStream::from(quote! {
        #[syscall_allowlist(#(#allowed_syscalls),*)]
        #fn_vis #fn_sig {
            // Enable syscall filtering
            serust_runtime::enable_syscall_filter(&[#(#allowed_syscalls),*]);

            let result = (|| #fn_block)();

            // Disable syscall filtering
            serust_runtime::disable_syscall_filter();

            result
        }
    })
}

/// Provenance tracking macro
#[proc_macro_attribute]
pub fn track_provenance(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();
    let fn_block = &input_fn.block;
    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;

    TokenStream::from(quote! {
        #fn_vis #fn_sig {
            // Start provenance tracking
            let execution_id = serust_runtime::start_execution_tracking(#fn_name_str);

            // Execute with tracking
            let result = (|| #fn_block)();

            // Complete provenance tracking
            serust_runtime::complete_execution_tracking(&execution_id);

            result
        }
    })
}

/// Test orbit compliance macro
#[proc_macro]
pub fn test_orbit_compliance(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as AttributeArgs);

    let mut function_name = String::new();
    let mut expected_orbit = String::new();

    for arg in args {
        match arg {
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("function") => {
                if let Lit::Str(lit_str) = nv.lit {
                    function_name = lit_str.value();
                }
            }
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("orbit") => {
                if let Lit::Str(lit_str) = nv.lit {
                    expected_orbit = lit_str.value();
                }
            }
            _ => {}
        }
    }

    TokenStream::from(quote! {
        #[test]
        fn test_orbit_compliance() {
            let actual_orbit = serust_runtime::get_function_orbit(#function_name);
            assert_eq!(actual_orbit, #expected_orbit,
                "Function {} should be in orbit {} but found in {}",
                #function_name, #expected_orbit, actual_orbit);
        }
    })
}

/// Friendly declarative security test macro
#[proc_macro]
pub fn security_test(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as AttributeArgs);

    let mut test_name = String::new();
    let mut domain = String::new();
    let mut should_allow = true;

    for arg in args {
        match arg {
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("name") => {
                if let Lit::Str(lit_str) = nv.lit {
                    test_name = lit_str.value();
                }
            }
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("domain") => {
                if let Lit::Str(lit_str) = nv.lit {
                    domain = lit_str.value();
                }
            }
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("should_allow") => {
                if let Lit::Bool(lit_bool) = nv.lit {
                    should_allow = lit_bool.value;
                }
            }
            _ => {}
        }
    }

    let test_fn_name = syn::Ident::new(&format!("test_{}", test_name.replace(" ", "_")), proc_macro2::Span::call_site());

    TokenStream::from(quote! {
        #[test]
        fn #test_fn_name() {
            let result = serust_runtime::test_domain_access(#domain, #test_name);
            if #should_allow {
                assert!(result.is_ok(), "Expected {} to be allowed in domain {}", #test_name, #domain);
            } else {
                assert!(result.is_err(), "Expected {} to be denied in domain {}", #test_name, #domain);
            }
        }
    })
}
