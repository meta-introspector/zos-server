use clippy_utils::diagnostics::{span_lint_and_help, span_lint_and_sugg};
use clippy_utils::source::snippet;
use rustc_hir::{Body, Expr, ExprKind, FnDecl, Item, ItemKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::Span;

declare_lint! {
    /// Detects operations that require LMFDB complexity analysis
    pub LMFDB_COMPLEXITY_AUDIT,
    Warn,
    "operations requiring LMFDB complexity orbit analysis"
}

declare_lint! {
    /// Detects missing complexity signature annotations
    pub MISSING_COMPLEXITY_SIGNATURE,
    Warn,
    "functions missing complexity signature annotations"
}

declare_lint! {
    /// Detects operations exceeding security context bounds
    pub SECURITY_CONTEXT_VIOLATION,
    Deny,
    "operations that exceed current security context complexity bounds"
}

declare_lint! {
    /// Detects functions that should have eigenvalue decomposition
    pub MISSING_EIGENVALUE_DECOMPOSITION,
    Warn,
    "functions that should be decomposed into structural eigenvalues"
}

declare_lint! {
    /// Detects potential novelty claims that need proof
    pub UNPROVEN_NOVELTY_CLAIM,
    Warn,
    "code claiming novelty without cryptographic proof"
}

declare_lint_pass!(Clip2SecureLints => [
    LMFDB_COMPLEXITY_AUDIT,
    MISSING_COMPLEXITY_SIGNATURE,
    SECURITY_CONTEXT_VIOLATION,
    MISSING_EIGENVALUE_DECOMPOSITION,
    UNPROVEN_NOVELTY_CLAIM
]);

impl<'tcx> LateLintPass<'tcx> for Clip2SecureLints {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: rustc_hir::intravisit::FnKind<'tcx>,
        _: &'tcx FnDecl<'_>,
        body: &'tcx Body<'_>,
        span: Span,
        def_id: rustc_span::def_id::LocalDefId,
    ) {
        let fn_name = cx.tcx.def_path_str(def_id.to_def_id());

        // Check for missing complexity signature
        if !self.has_complexity_signature(cx, def_id) {
            span_lint_and_help(
                cx,
                MISSING_COMPLEXITY_SIGNATURE,
                span,
                &format!("function '{}' missing complexity signature", fn_name),
                None,
                "Add #[complexity(level = \"Medium\", orbit_size = 1000)] annotation",
            );
        }

        // Analyze function body for complexity patterns
        self.analyze_function_complexity(cx, body, span, &fn_name);
    }

    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) {
        match &expr.kind {
            // Detect loops that need complexity analysis
            ExprKind::Loop(_, _, _, _) => {
                span_lint_and_help(
                    cx,
                    LMFDB_COMPLEXITY_AUDIT,
                    expr.span,
                    "loop detected - requires LMFDB complexity orbit analysis",
                    None,
                    "Add #[lmfdb_orbit(size = N)] annotation with mathematical proof",
                );
            }

            // Detect recursive calls
            ExprKind::Call(func, _) => {
                if self.is_potentially_recursive(cx, func) {
                    span_lint_and_help(
                        cx,
                        LMFDB_COMPLEXITY_AUDIT,
                        expr.span,
                        "potentially recursive call - requires complexity bounds",
                        None,
                        "Verify termination and add complexity signature",
                    );
                }
            }

            // Detect unsafe blocks
            ExprKind::Block(block, _) if block.rules.is_unsafe() => {
                span_lint_and_help(
                    cx,
                    SECURITY_CONTEXT_VIOLATION,
                    expr.span,
                    "unsafe block requires security context verification",
                    None,
                    "Ensure current security context allows unsafe operations",
                );
            }

            _ => {}
        }
    }

    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx Item<'_>) {
        match &item.kind {
            // Check for novelty claims in documentation
            ItemKind::Fn(_, _, _) | ItemKind::Struct(_, _) | ItemKind::Enum(_, _) => {
                if self.has_novelty_claim(cx, item) && !self.has_novelty_proof(cx, item) {
                    span_lint_and_help(
                        cx,
                        UNPROVEN_NOVELTY_CLAIM,
                        item.span,
                        "code claims novelty without cryptographic proof",
                        None,
                        "Add #[novelty_proof(hash = \"abc123\")] with verification",
                    );
                }
            }
            _ => {}
        }
    }
}

impl Clip2SecureLints {
    fn has_complexity_signature(
        &self,
        cx: &LateContext<'_>,
        def_id: rustc_span::def_id::LocalDefId,
    ) -> bool {
        // Check if function has complexity annotation
        let attrs = cx.tcx.hir().attrs(cx.tcx.local_def_id_to_hir_id(def_id));
        attrs.iter().any(|attr| {
            attr.path()
                .segments
                .last()
                .map_or(false, |seg| seg.ident.name.as_str() == "complexity")
        })
    }

    fn analyze_function_complexity(
        &self,
        cx: &LateContext<'_>,
        body: &Body<'_>,
        span: Span,
        fn_name: &str,
    ) {
        let mut complexity_analyzer = ComplexityAnalyzer::new();
        complexity_analyzer.visit_body(body);

        if complexity_analyzer.estimated_orbit_size > 10000 {
            span_lint_and_help(
                cx,
                LMFDB_COMPLEXITY_AUDIT,
                span,
                &format!(
                    "function '{}' has high complexity orbit ({})",
                    fn_name, complexity_analyzer.estimated_orbit_size
                ),
                None,
                "Consider breaking into smaller functions or add complexity proof",
            );
        }

        if complexity_analyzer.needs_eigenvalue_decomposition {
            span_lint_and_help(
                cx,
                MISSING_EIGENVALUE_DECOMPOSITION,
                span,
                &format!(
                    "function '{}' should be decomposed into eigenvalues",
                    fn_name
                ),
                None,
                "Add #[eigenvalue_decomposition] annotation",
            );
        }
    }

    fn is_potentially_recursive(&self, _cx: &LateContext<'_>, _func: &Expr<'_>) -> bool {
        // Simplified check - in practice would analyze call graph
        true
    }

    fn has_novelty_claim(&self, cx: &LateContext<'_>, item: &Item<'_>) -> bool {
        // Check documentation for novelty claims
        let attrs = cx.tcx.hir().attrs(item.hir_id());
        attrs.iter().any(|attr| {
            if let Some(doc) = attr.doc_str() {
                let doc_str = doc.as_str().to_lowercase();
                doc_str.contains("novel")
                    || doc_str.contains("new")
                    || doc_str.contains("innovative")
            } else {
                false
            }
        })
    }

    fn has_novelty_proof(&self, cx: &LateContext<'_>, item: &Item<'_>) -> bool {
        // Check for novelty proof annotation
        let attrs = cx.tcx.hir().attrs(item.hir_id());
        attrs.iter().any(|attr| {
            attr.path()
                .segments
                .last()
                .map_or(false, |seg| seg.ident.name.as_str() == "novelty_proof")
        })
    }
}

struct ComplexityAnalyzer {
    estimated_orbit_size: u64,
    loop_nesting_depth: u32,
    recursion_depth: u32,
    needs_eigenvalue_decomposition: bool,
}

impl ComplexityAnalyzer {
    fn new() -> Self {
        Self {
            estimated_orbit_size: 1,
            loop_nesting_depth: 0,
            recursion_depth: 0,
            needs_eigenvalue_decomposition: false,
        }
    }

    fn visit_body(&mut self, body: &Body<'_>) {
        self.visit_expr(&body.value);

        // Determine if function needs eigenvalue decomposition
        if self.estimated_orbit_size > 1000 || self.loop_nesting_depth > 2 {
            self.needs_eigenvalue_decomposition = true;
        }
    }

    fn visit_expr(&mut self, expr: &Expr<'_>) {
        match &expr.kind {
            ExprKind::Loop(body, _, _, _) => {
                self.loop_nesting_depth += 1;
                self.estimated_orbit_size *= 100; // Assume 100 iterations
                self.visit_block(body);
                self.loop_nesting_depth -= 1;
            }
            ExprKind::Call(_, _) => {
                self.recursion_depth += 1;
                self.estimated_orbit_size *= 10; // Assume 10x complexity for calls
            }
            ExprKind::Block(block, _) => {
                self.visit_block(block);
            }
            _ => {
                // Visit child expressions
                rustc_hir::intravisit::walk_expr(self, expr);
            }
        }
    }

    fn visit_block(&mut self, block: &rustc_hir::Block<'_>) {
        for stmt in block.stmts {
            if let rustc_hir::StmtKind::Expr(expr) | rustc_hir::StmtKind::Semi(expr) = &stmt.kind {
                self.visit_expr(expr);
            }
        }
        if let Some(expr) = block.expr {
            self.visit_expr(expr);
        }
    }
}

impl<'tcx> rustc_hir::intravisit::Visitor<'tcx> for ComplexityAnalyzer {
    type NestedFilter = rustc_middle::hir::nested_filter::None;

    fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
        self.visit_expr(expr);
    }
}
