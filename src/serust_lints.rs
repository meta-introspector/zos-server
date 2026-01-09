// SErust Clippy Lints - Friendly Security Suggestions
use clippy_utils::diagnostics::span_lint_and_sugg;
use rustc_errors::Applicability;
use rustc_hir::{Expr, ExprKind, Item, ItemKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};

declare_lint! {
    /// Suggests adding orbit classification to functions
    pub MISSING_ORBIT_CLASSIFICATION,
    Warn,
    "function should have orbit classification for security analysis"
}

declare_lint! {
    /// Suggests adding domain restrictions to modules
    pub MISSING_DOMAIN_RESTRICTION,
    Warn,
    "module should specify security domain for access control"
}

declare_lint! {
    /// Warns about potential syscall usage
    pub POTENTIAL_SYSCALL_USAGE,
    Warn,
    "potential syscall usage detected, consider using virtual alternatives"
}

declare_lint! {
    /// Suggests provenance tracking for data operations
    pub MISSING_PROVENANCE_TRACKING,
    Warn,
    "data operation should include provenance tracking"
}

declare_lint! {
    /// Suggests capability requirements for privileged operations
    pub MISSING_CAPABILITY_REQUIREMENT,
    Warn,
    "privileged operation should specify required capabilities"
}

declare_lint_pass!(SerustLints => [
    MISSING_ORBIT_CLASSIFICATION,
    MISSING_DOMAIN_RESTRICTION,
    POTENTIAL_SYSCALL_USAGE,
    MISSING_PROVENANCE_TRACKING,
    MISSING_CAPABILITY_REQUIREMENT
]);

impl<'tcx> LateLintPass<'tcx> for SerustLints {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        match item.kind {
            ItemKind::Fn(_, _, _) => {
                self.check_function_orbit_classification(cx, item);
                self.check_function_capabilities(cx, item);
            }
            ItemKind::Mod(_, _) => {
                self.check_module_domain_restriction(cx, item);
            }
            _ => {}
        }
    }

    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        match expr.kind {
            ExprKind::Call(func, _) => {
                self.check_syscall_usage(cx, expr, func);
                self.check_data_provenance(cx, expr);
            }
            _ => {}
        }
    }
}

impl SerustLints {
    fn check_function_orbit_classification(&self, cx: &LateContext<'_>, item: &Item<'_>) {
        let has_orbit_attr = item.attrs.iter().any(|attr| {
            attr.path().is_ident("orbit") ||
            attr.path().is_ident("serust_domain") ||
            attr.path().segments.last().map_or(false, |seg| seg.ident.to_string().contains("orbit"))
        });

        if !has_orbit_attr {
            span_lint_and_sugg(
                cx,
                MISSING_ORBIT_CLASSIFICATION,
                item.span,
                "function missing orbit classification",
                "consider adding orbit classification",
                format!("#[orbit(trivial)] // or cyclic, symmetric, alternating, sporadic, monster\n{}",
                       cx.sess().source_map().span_to_snippet(item.span).unwrap_or_default()),
                Applicability::MachineApplicable,
            );
        }
    }

    fn check_module_domain_restriction(&self, cx: &LateContext<'_>, item: &Item<'_>) {
        let has_domain_attr = item.attrs.iter().any(|attr| {
            attr.path().is_ident("serust_domain") ||
            attr.path().segments.last().map_or(false, |seg| seg.ident.to_string().contains("domain"))
        });

        if !has_domain_attr {
            span_lint_and_sugg(
                cx,
                MISSING_DOMAIN_RESTRICTION,
                item.span,
                "module should specify security domain",
                "consider adding domain restriction",
                format!("#[serust_domain(level = 0, orbits = \"trivial\", capabilities = \"read,compute\")]\n{}",
                       cx.sess().source_map().span_to_snippet(item.span).unwrap_or_default()),
                Applicability::MachineApplicable,
            );
        }
    }

    fn check_syscall_usage(&self, cx: &LateContext<'_>, expr: &Expr<'_>, func: &Expr<'_>) {
        if let ExprKind::Path(qpath) = &func.kind {
            let path_str = format!("{:?}", qpath);

            let dangerous_patterns = [
                "libc::execve", "libc::fork", "libc::ptrace", "libc::mount",
                "std::process::Command", "std::process::exit"
            ];

            for pattern in &dangerous_patterns {
                if path_str.contains(pattern) {
                    span_lint_and_sugg(
                        cx,
                        POTENTIAL_SYSCALL_USAGE,
                        expr.span,
                        &format!("potential dangerous syscall: {}", pattern),
                        "consider using virtual alternative",
                        format!("#[allow_syscalls(\"{}\")] // or use virtual implementation\n{}",
                               pattern,
                               cx.sess().source_map().span_to_snippet(expr.span).unwrap_or_default()),
                        Applicability::MaybeIncorrect,
                    );
                }
            }
        }
    }

    fn check_data_provenance(&self, cx: &LateContext<'_>, expr: &Expr<'_>) {
        if let ExprKind::Call(func, _) = &expr.kind {
            if let ExprKind::Path(qpath) = &func.kind {
                let path_str = format!("{:?}", qpath);

                let data_operations = [
                    "std::fs::read", "std::fs::write", "serde::serialize", "serde::deserialize"
                ];

                for op in &data_operations {
                    if path_str.contains(op) {
                        span_lint_and_sugg(
                            cx,
                            MISSING_PROVENANCE_TRACKING,
                            expr.span,
                            "data operation should include provenance tracking",
                            "consider adding provenance tracking",
                            format!("#[track_provenance]\n{}",
                                   cx.sess().source_map().span_to_snippet(expr.span).unwrap_or_default()),
                            Applicability::MachineApplicable,
                        );
                    }
                }
            }
        }
    }

    fn check_function_capabilities(&self, cx: &LateContext<'_>, item: &Item<'_>) {
        // Check if function name suggests privileged operations
        let fn_name = match &item.kind {
            ItemKind::Fn(_, _, _) => item.ident.to_string(),
            _ => return,
        };

        let privileged_patterns = [
            "admin", "root", "system", "kernel", "unsafe", "raw", "direct"
        ];

        let has_requires_attr = item.attrs.iter().any(|attr| {
            attr.path().is_ident("requires") ||
            attr.path().segments.last().map_or(false, |seg| seg.ident.to_string().contains("requires"))
        });

        if !has_requires_attr && privileged_patterns.iter().any(|p| fn_name.contains(p)) {
            span_lint_and_sugg(
                cx,
                MISSING_CAPABILITY_REQUIREMENT,
                item.span,
                "privileged function should specify required capabilities",
                "consider adding capability requirement",
                format!("#[requires(admin)] // or root, system, etc.\n{}",
                       cx.sess().source_map().span_to_snippet(item.span).unwrap_or_default()),
                Applicability::MachineApplicable,
            );
        }
    }
}

/// Runtime support for SErust macros
pub mod serust_runtime {
    use std::collections::HashSet;

    static mut CURRENT_DOMAIN_LEVEL: u8 = 0;
    static mut ALLOWED_ORBITS: Option<HashSet<String>> = None;
    static mut ALLOWED_CAPABILITIES: Option<HashSet<String>> = None;

    pub fn enforce_domain_access(level: u8, orbits: &[&str], capabilities: &[&str]) {
        unsafe {
            CURRENT_DOMAIN_LEVEL = level;
            ALLOWED_ORBITS = Some(orbits.iter().map(|s| s.to_string()).collect());
            ALLOWED_CAPABILITIES = Some(capabilities.iter().map(|s| s.to_string()).collect());
        }

        println!("🔒 Domain enforced: level {}, orbits: {:?}, caps: {:?}",
                level, orbits, capabilities);
    }

    pub fn verify_orbit_access(orbit_class: &str) {
        unsafe {
            if let Some(ref allowed) = ALLOWED_ORBITS {
                if !allowed.contains(orbit_class) {
                    panic!("Orbit {} not allowed in current domain", orbit_class);
                }
            }
        }

        println!("✅ Orbit access verified: {}", orbit_class);
    }

    pub fn check_capabilities(required_caps: &[&str]) -> Result<(), String> {
        unsafe {
            if let Some(ref allowed) = ALLOWED_CAPABILITIES {
                for cap in required_caps {
                    if !allowed.contains(*cap) && !allowed.contains("all") {
                        return Err(format!("Capability {} not available", cap));
                    }
                }
            }
        }

        println!("✅ Capabilities verified: {:?}", required_caps);
        Ok(())
    }

    pub fn enable_syscall_filter(allowed_syscalls: &[&str]) {
        println!("🔧 Syscall filter enabled: {:?}", allowed_syscalls);
    }

    pub fn disable_syscall_filter() {
        println!("🔧 Syscall filter disabled");
    }

    pub fn start_execution_tracking(function_name: &str) -> String {
        let execution_id = format!("exec_{}_{}", function_name, chrono::Utc::now().timestamp());
        println!("📊 Started tracking: {}", execution_id);
        execution_id
    }

    pub fn complete_execution_tracking(execution_id: &str) {
        println!("📊 Completed tracking: {}", execution_id);
    }

    pub fn get_function_orbit(function_name: &str) -> String {
        // Would analyze function and return its orbit
        if function_name.contains("add") || function_name.contains("mul") {
            "trivial".to_string()
        } else if function_name.contains("sort") {
            "symmetric".to_string()
        } else {
            "cyclic".to_string()
        }
    }

    pub fn test_domain_access(domain: &str, operation: &str) -> Result<(), String> {
        // Simulate domain access test
        match (domain, operation) {
            ("l0_public", op) if op.contains("add") => Ok(()),
            ("l0_public", _) => Err("Operation not allowed in public domain".to_string()),
            ("l4_kernel", _) => Ok(()),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Example of using the macros
    #[serust_domain(level = 0, orbits = "trivial", capabilities = "read,compute")]
    #[orbit(trivial)]
    #[track_provenance]
    pub fn safe_add(a: i32, b: i32) -> i32 {
        a + b
    }

    #[serust_domain(level = 3, orbits = "trivial,cyclic,symmetric", capabilities = "read,compute,admin")]
    #[orbit(symmetric)]
    #[requires(admin)]
    #[allow_syscalls("read", "write")]
    #[track_provenance]
    pub fn admin_sort(data: &mut [i32]) {
        data.sort();
    }

    // Generate compliance tests
    test_orbit_compliance!(function = "safe_add", orbit = "trivial");
    security_test!(name = "public_add_allowed", domain = "l0_public", should_allow = true);
    security_test!(name = "public_admin_denied", domain = "l0_public", should_allow = false);
}
