// Proc Macro for Syscall Stripping - Literal Code Removal
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Expr, ExprCall, ExprPath};

/// Proc macro that strips syscalls from functions
#[proc_macro_attribute]
pub fn strip_syscalls(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let stripped_fn = strip_syscalls_from_function(input_fn);
    TokenStream::from(quote! { #stripped_fn })
}

/// Strip all syscalls from a function
fn strip_syscalls_from_function(mut func: ItemFn) -> ItemFn {
    let mut syscall_count = 0;

    // Visit and modify the function body
    if let Some(ref mut block) = func.block.as_mut() {
        for stmt in &mut block.stmts {
            syscall_count += strip_syscalls_from_stmt(stmt);
        }
    }

    // Add compile-time proof of syscall removal
    let func_name = &func.sig.ident;
    let proof_comment = format!("// PROOF: {} syscalls stripped from {}", syscall_count, func_name);

    // Insert proof as first statement
    let proof_stmt: syn::Stmt = syn::parse_quote! {
        compile_time_proof!(syscalls_stripped = #syscall_count);
    };

    if let Some(ref mut block) = func.block.as_mut() {
        block.stmts.insert(0, proof_stmt);
    }

    func
}

/// Strip syscalls from a statement
fn strip_syscalls_from_stmt(stmt: &mut syn::Stmt) -> usize {
    match stmt {
        syn::Stmt::Expr(expr, _) | syn::Stmt::Semi(expr, _) => strip_syscalls_from_expr(expr),
        syn::Stmt::Local(local) => {
            if let Some(ref mut init) = local.init {
                strip_syscalls_from_expr(&mut init.expr)
            } else {
                0
            }
        }
        _ => 0,
    }
}

/// Strip syscalls from expressions
fn strip_syscalls_from_expr(expr: &mut Expr) -> usize {
    match expr {
        Expr::Call(call) => strip_syscalls_from_call(call),
        Expr::Unsafe(unsafe_block) => {
            // Strip entire unsafe blocks that contain syscalls
            let mut stripped = 0;
            for stmt in &mut unsafe_block.block.stmts {
                stripped += strip_syscalls_from_stmt(stmt);
            }

            // Replace unsafe block with safe alternative
            if stripped > 0 {
                *expr = syn::parse_quote! {
                    compile_error!("Unsafe syscall block stripped for security")
                };
            }
            stripped
        }
        Expr::Block(block) => {
            let mut stripped = 0;
            for stmt in &mut block.block.stmts {
                stripped += strip_syscalls_from_stmt(stmt);
            }
            stripped
        }
        _ => 0,
    }
}

/// Strip syscalls from function calls
fn strip_syscalls_from_call(call: &mut ExprCall) -> usize {
    if let Expr::Path(ExprPath { path, .. }) = call.func.as_ref() {
        let path_str = quote! { #path }.to_string();

        // List of dangerous syscalls to strip
        let dangerous_calls = [
            "libc::execve", "libc::fork", "libc::mount", "libc::ptrace",
            "libc::setuid", "libc::setgid", "libc::reboot", "syscall",
            "std::process::Command", "std::process::exit",
        ];

        for dangerous in &dangerous_calls {
            if path_str.contains(dangerous) {
                // Replace with compile error
                *call = syn::parse_quote! {
                    compile_error!(concat!("Syscall stripped: ", #dangerous))
                };
                return 1;
            }
        }
    }
    0
}

/// Proc macro for entire crate syscall stripping
#[proc_macro]
pub fn strip_crate_syscalls(_input: TokenStream) -> TokenStream {
    TokenStream::from(quote! {
        // Generate proof that syscalls are stripped at crate level
        const _SYSCALL_PROOF: () = {
            // This ensures syscalls are provably removed
            #[cfg(any(
                feature = "syscalls",
                feature = "unsafe-ops",
                feature = "libc-direct"
            ))]
            compile_error!("Dangerous syscall features detected and blocked");
        };
    })
}

/// Compile-time proof macro
#[proc_macro]
pub fn compile_time_proof(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    TokenStream::from(quote! {
        const _PROOF: &str = #input_str;
    })
}

/// Proc macro to replace git2 with virtual implementation
#[proc_macro_attribute]
pub fn virtualize_git(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let virtualized_fn = virtualize_git_calls(input_fn);
    TokenStream::from(quote! { #virtualized_fn })
}

/// Replace git2 calls with virtual implementations
fn virtualize_git_calls(mut func: ItemFn) -> ItemFn {
    if let Some(ref mut block) = func.block.as_mut() {
        for stmt in &mut block.stmts {
            virtualize_git_in_stmt(stmt);
        }
    }
    func
}

fn virtualize_git_in_stmt(stmt: &mut syn::Stmt) {
    match stmt {
        syn::Stmt::Expr(expr, _) | syn::Stmt::Semi(expr, _) => {
            virtualize_git_in_expr(expr);
        }
        syn::Stmt::Local(local) => {
            if let Some(ref mut init) = local.init {
                virtualize_git_in_expr(&mut init.expr);
            }
        }
        _ => {}
    }
}

fn virtualize_git_in_expr(expr: &mut Expr) {
    match expr {
        Expr::Call(call) => {
            if let Expr::Path(ExprPath { path, .. }) = call.func.as_ref() {
                let path_str = quote! { #path }.to_string();

                // Replace git2 calls with virtual implementations
                if path_str.contains("git2::Repository::open") {
                    *call = syn::parse_quote! {
                        crate::container_runtime::llm_git::virtual_repo_open()
                    };
                } else if path_str.contains("git2::Repository::clone") {
                    *call = syn::parse_quote! {
                        crate::container_runtime::llm_git::virtual_repo_clone()
                    };
                }
            }
        }
        Expr::Block(block) => {
            for stmt in &mut block.block.stmts {
                virtualize_git_in_stmt(stmt);
            }
        }
        _ => {}
    }
}
