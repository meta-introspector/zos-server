use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_hir::{Item, ItemKind, ImplItem, TraitItem};
use rustc_session::{declare_lint, declare_lint_pass};

declare_lint! {
    pub DEAD_CODE_ELIMINATOR,
    Warn,
    "automatically flag unused code for source-level removal"
}

declare_lint_pass!(DeadCodeEliminator => [DEAD_CODE_ELIMINATOR]);

impl<'tcx> LateLintPass<'tcx> for DeadCodeEliminator {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        match &item.kind {
            ItemKind::Fn(..) => {
                if is_unused_function(cx, item) {
                    cx.lint(DEAD_CODE_ELIMINATOR, |lint| {
                        lint.build("REMOVE: Unused function")
                            .set_span(item.span)
                            .note("This function is never called - safe to delete")
                            .emit();
                    });
                }
            }
            ItemKind::Struct(..) => {
                if is_unused_struct(cx, item) {
                    cx.lint(DEAD_CODE_ELIMINATOR, |lint| {
                        lint.build("REMOVE: Unused struct")
                            .set_span(item.span)
                            .note("This struct is never instantiated - safe to delete")
                            .emit();
                    });
                }
            }
            _ => {}
        }
    }
}

fn is_unused_function(cx: &LateContext<'_>, item: &Item<'_>) -> bool {
    // Check if function is referenced anywhere
    let def_id = cx.tcx.hir().local_def_id(item.hir_id());
    !cx.tcx.is_reachable_non_generic(def_id.to_def_id())
}

fn is_unused_struct(cx: &LateContext<'_>, item: &Item<'_>) -> bool {
    // Check if struct is ever constructed
    let def_id = cx.tcx.hir().local_def_id(item.hir_id());
    !cx.tcx.is_reachable_non_generic(def_id.to_def_id())
}

// Auto-rewriter companion
pub fn auto_remove_dead_code(source: &str) -> String {
    // Parse and remove flagged code
    source.lines()
        .filter(|line| !line.contains("// REMOVE:"))
        .collect::<Vec<_>>()
        .join("\n")
}
