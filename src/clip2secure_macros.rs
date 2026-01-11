use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemFn, Lit, Meta, NestedMeta};

/// Complexity signature annotation for functions
///
/// # Example
/// ```rust
/// #[complexity(level = "Medium", orbit_size = 1000, time = "O(n)", space = "O(1)")]
/// fn my_function() {
///     // Implementation
/// }
/// ```
#[proc_macro_attribute]
pub fn complexity(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input_fn = parse_macro_input!(input as ItemFn);

    let mut level = String::from("Unknown");
    let mut orbit_size = 0u64;
    let mut time_complexity = String::from("Unknown");
    let mut space_complexity = String::from("Unknown");

    // Parse arguments
    for arg in args {
        match arg {
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("level") => {
                if let Lit::Str(lit_str) = nv.lit {
                    level = lit_str.value();
                }
            }
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("orbit_size") => {
                if let Lit::Int(lit_int) = nv.lit {
                    orbit_size = lit_int.base10_parse().unwrap_or(0);
                }
            }
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("time") => {
                if let Lit::Str(lit_str) = nv.lit {
                    time_complexity = lit_str.value();
                }
            }
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("space") => {
                if let Lit::Str(lit_str) = nv.lit {
                    space_complexity = lit_str.value();
                }
            }
            _ => {}
        }
    }

    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();

    let expanded = quote! {
        #[doc = concat!("Complexity: ", #level, " - Orbit Size: ", #orbit_size, " - Time: ", #time_complexity, " - Space: ", #space_complexity)]
        #input_fn

        // Generate complexity metadata
        const _: () = {
            #[allow(non_upper_case_globals)]
            static #fn_name: crate::ComplexitySignature = crate::ComplexitySignature {
                function_name: #fn_name_str,
                level: #level,
                orbit_size: #orbit_size,
                time_complexity: #time_complexity,
                space_complexity: #space_complexity,
            };
        };
    };

    TokenStream::from(expanded)
}

/// LMFDB orbit annotation for code blocks
///
/// # Example
/// ```rust
/// #[lmfdb_orbit(size = 1000, class = "P", proof_hash = "abc123")]
/// fn complex_algorithm() {
///     // Implementation with proven complexity bounds
/// }
/// ```
#[proc_macro_attribute]
pub fn lmfdb_orbit(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input_fn = parse_macro_input!(input as ItemFn);

    let mut orbit_size = 0u64;
    let mut complexity_class = String::from("Unknown");
    let mut proof_hash = String::from("");

    // Parse arguments
    for arg in args {
        match arg {
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("size") => {
                if let Lit::Int(lit_int) = nv.lit {
                    orbit_size = lit_int.base10_parse().unwrap_or(0);
                }
            }
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("class") => {
                if let Lit::Str(lit_str) = nv.lit {
                    complexity_class = lit_str.value();
                }
            }
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("proof_hash") => {
                if let Lit::Str(lit_str) = nv.lit {
                    proof_hash = lit_str.value();
                }
            }
            _ => {}
        }
    }

    let fn_name = &input_fn.sig.ident;

    let expanded = quote! {
        #input_fn

        // Generate LMFDB orbit metadata
        const _: () = {
            #[allow(non_upper_case_globals)]
            static #fn_name: crate::LMFDBOrbit = crate::LMFDBOrbit {
                orbit_size: #orbit_size,
                complexity_class: #complexity_class,
                proof_hash: #proof_hash,
            };
        };
    };

    TokenStream::from(expanded)
}

/// Eigenvalue decomposition annotation
///
/// # Example
/// ```rust
/// #[eigenvalue_decomposition(real = 2.5, imaginary = 1.0, structural_meaning = "transformation")]
/// fn matrix_operation() -> Matrix {
///     // Implementation that can be decomposed into eigenvalues
/// }
/// ```
#[proc_macro_attribute]
pub fn eigenvalue_decomposition(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input_fn = parse_macro_input!(input as ItemFn);

    let mut real_part = 0.0f64;
    let mut imaginary_part = 0.0f64;
    let mut structural_meaning = String::from("unknown");

    // Parse arguments
    for arg in args {
        match arg {
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("real") => {
                if let Lit::Float(lit_float) = nv.lit {
                    real_part = lit_float.base10_parse().unwrap_or(0.0);
                }
            }
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("imaginary") => {
                if let Lit::Float(lit_float) = nv.lit {
                    imaginary_part = lit_float.base10_parse().unwrap_or(0.0);
                }
            }
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("structural_meaning") => {
                if let Lit::Str(lit_str) = nv.lit {
                    structural_meaning = lit_str.value();
                }
            }
            _ => {}
        }
    }

    let fn_name = &input_fn.sig.ident;

    let expanded = quote! {
        #input_fn

        // Generate eigenvalue metadata
        const _: () = {
            #[allow(non_upper_case_globals)]
            static #fn_name: crate::StructuralEigenvalue = crate::StructuralEigenvalue {
                real_part: #real_part,
                imaginary_part: #imaginary_part,
                magnitude: (#real_part * #real_part + #imaginary_part * #imaginary_part).sqrt(),
                structural_meaning: #structural_meaning,
            };
        };
    };

    TokenStream::from(expanded)
}

/// Novelty proof annotation for claiming innovation
///
/// # Example
/// ```rust
/// #[novelty_proof(hash = "abc123", proof_type = "ZeroKnowledge", novelty_score = 0.95)]
/// fn novel_algorithm() {
///     // Implementation with proven novelty
/// }
/// ```
#[proc_macro_attribute]
pub fn novelty_proof(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input_fn = parse_macro_input!(input as ItemFn);

    let mut proof_hash = String::from("");
    let mut proof_type = String::from("Unknown");
    let mut novelty_score = 0.0f64;

    // Parse arguments
    for arg in args {
        match arg {
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("hash") => {
                if let Lit::Str(lit_str) = nv.lit {
                    proof_hash = lit_str.value();
                }
            }
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("proof_type") => {
                if let Lit::Str(lit_str) = nv.lit {
                    proof_type = lit_str.value();
                }
            }
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("novelty_score") => {
                if let Lit::Float(lit_float) = nv.lit {
                    novelty_score = lit_float.base10_parse().unwrap_or(0.0);
                }
            }
            _ => {}
        }
    }

    let fn_name = &input_fn.sig.ident;

    let expanded = quote! {
        #input_fn

        // Generate novelty proof metadata
        const _: () = {
            #[allow(non_upper_case_globals)]
            static #fn_name: crate::NoveltyProof = crate::NoveltyProof {
                proof_hash: #proof_hash,
                proof_type: #proof_type,
                novelty_score: #novelty_score,
                economic_value: #novelty_score * 1000.0, // Base value calculation
            };
        };
    };

    TokenStream::from(expanded)
}

/// Security context annotation for access control
///
/// # Example
/// ```rust
/// #[security_context(level = "Admin", price_tier = 10000.0, matrix_access = "UpperTriangular")]
/// fn admin_function() {
///     // Implementation requiring admin access
/// }
/// ```
#[proc_macro_attribute]
pub fn security_context(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input_fn = parse_macro_input!(input as ItemFn);

    let mut security_level = String::from("Public");
    let mut price_tier = 0.0f64;
    let mut matrix_access = String::from("DiagonalOnly");

    // Parse arguments
    for arg in args {
        match arg {
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("level") => {
                if let Lit::Str(lit_str) = nv.lit {
                    security_level = lit_str.value();
                }
            }
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("price_tier") => {
                if let Lit::Float(lit_float) = nv.lit {
                    price_tier = lit_float.base10_parse().unwrap_or(0.0);
                }
            }
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("matrix_access") => {
                if let Lit::Str(lit_str) = nv.lit {
                    matrix_access = lit_str.value();
                }
            }
            _ => {}
        }
    }

    let fn_name = &input_fn.sig.ident;

    let expanded = quote! {
        #input_fn

        // Generate security context metadata
        const _: () = {
            #[allow(non_upper_case_globals)]
            static #fn_name: crate::SecurityContext = crate::SecurityContext {
                security_level: #security_level,
                price_tier: #price_tier,
                matrix_access: #matrix_access,
            };
        };
    };

    TokenStream::from(expanded)
}
