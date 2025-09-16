// my_lint/src/lib.rs  (Dylint project)
#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_hir;
extern crate rustc_span;

use rustc_hir as hir;
use rustc_hir::def::{DefKind, Res};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_span::{Span, Symbol};
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Instant;

static START_INSTANT: OnceLock<Instant> = OnceLock::new();

fn timestamp_prefix() -> String {
    let start = START_INSTANT.get_or_init(Instant::now);
    let elapsed = start.elapsed();
    format!("[{:>6}.{:03}s]", elapsed.as_secs(), elapsed.subsec_millis())
}

macro_rules! eprintln_ts {
    ($($arg:tt)*) => {{
        eprintln!("{} {}", crate::timestamp_prefix(), format!($($arg)*));
    }};
}

// Use dylint's impl_late_lint! macro to allow a custom struct
dylint_linting::impl_late_lint! {
    /// ### What it does
    /// Checks whether a realtime function (marked rt:realtime in main.rs) calls a non-realtime function
    ///
    /// ### Why is this bad?
    /// Realtime functions should only call other realtime functions to preserve realtime performance characteristics
    ///
    /// ### Known problems
    /// None.
    ///
    /// ### Example
    ///
    /// ```rust
    /// /// rt:realtime
    /// fn realtime_fn() {
    ///     non_realtime_fn(); // This will trigger a warning
    /// }
    ///
    /// fn non_realtime_fn() {
    ///     // Function not marked as realtime
    /// }
    /// ```
    pub REALTIME_CALLS_NONREALTIME,
    Warn,
    "a realtime function (in main.rs) calls a non-realtime function",
    RealtimeCallsNonrealtime::new()  // Pass our custom struct instance
}

#[derive(Default)]
pub struct RealtimeCallsNonrealtime {
    // Whether we are inside the 'realtime function body' in main.rs (used as a stack)
    in_realtime_main_fn: Vec<hir::HirId>,
    // Record realtime property for closures bound by let: variable name -> is realtime
    closure_var_realtime: HashMap<String, bool>,
    // Record realtime property for function-pointer variables bound by let: variable name -> is realtime
    fnptr_var_realtime: HashMap<String, bool>,
}

impl RealtimeCallsNonrealtime {
    pub fn new() -> Self {
        Self {
            in_realtime_main_fn: Vec::new(),
            closure_var_realtime: HashMap::new(),
            fnptr_var_realtime: HashMap::new(),
        }
    }

    fn in_realtime(&self) -> bool {
        !self.in_realtime_main_fn.is_empty()
    }

    /// Read the doc marker on a function:
    ///  - rt:realtime            => Some(true)
    ///  - rt:nonrealtime:...     => Some(false)
    ///  - not marked             => None
    fn doc_marker_is_realtime(_cx: &LateContext<'_>, attrs: &[hir::Attribute]) -> Option<bool> {
        for attr in attrs {
            // Check doc comment attributes
            if attr.has_name(Symbol::intern("doc")) {
                if let Some(content) = attr.value_str() {
                    let doc_str = content.as_str();
                    if doc_str.contains("rt:realtime") {
                        return Some(true);
                    }
                    if doc_str.contains("rt:non_realtime") {
                        return Some(false);
                    }
                }
            }
        }
        None
    }

    /// Parse our injected call-info doc marker from attributes:
    /// Like: #[doc = "rt:call-info:<name>:<realtime|nonrealtime>"]
    /// Returns (name, is_realtime)
    fn extract_call_info_from_doc_attrs(attrs: &[hir::Attribute]) -> Option<(String, bool)> {
        for attr in attrs {
            if attr.has_name(Symbol::intern("doc")) {
                if let Some(content) = attr.value_str() {
                    let s = content.as_str();
                    if let Some(rest) = s.strip_prefix("rt:call-info:") {
                        let mut it = rest.split(':');
                        let name = it.next().unwrap_or("").to_string();
                        let rt = it.next().unwrap_or("");
                        let is_rt = matches!(rt, "realtime");
                        return Some((name, is_rt));
                    }
                }
            }
        }
        None
    }

    // Realtime determination for ordinary functions and traits
    fn callee_is_realtime(cx: &LateContext<'_>, def_id: rustc_hir::def_id::DefId) -> Option<bool> {
        // 首先检查是否为本地定义
        if !def_id.is_local() {
            // External crate: treat as unmarked (could be relaxed)
            eprintln_ts!(
                "External DefId: {:?}, Path: {}",
                def_id,
                cx.tcx.def_path_str(def_id)
            );
            return None;
        }

        let hir_id = cx.tcx.local_def_id_to_hir_id(def_id.expect_local());
        let attrs = cx.tcx.hir().attrs(hir_id);
        // eprintln!("Local DefId: {:?}, Path: {}", def_id, cx.tcx.def_path_str(def_id));
        // eprintln!("Attrs: {:?}", attrs);

        Self::doc_marker_is_realtime(cx, attrs)
    }
}

impl<'tcx> LateLintPass<'tcx> for RealtimeCallsNonrealtime {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _fk: rustc_hir::intravisit::FnKind<'tcx>,
        _decl: &'tcx hir::FnDecl<'tcx>,
        _body: &'tcx hir::Body<'tcx>,
        _span: Span,
        def_id: rustc_span::def_id::LocalDefId,
    ) {
        eprintln_ts!("[debug] check fn: {:?}", def_id);
        // Skip closures directly
        let did = def_id.to_def_id();
        if cx.tcx.def_kind(did) == DefKind::Closure {
            return;
        }
        self.in_realtime_main_fn.clear();
        let hir_id = cx.tcx.local_def_id_to_hir_id(def_id);
        let attrs = cx.tcx.hir().attrs(hir_id);
        if matches!(Self::doc_marker_is_realtime(cx, attrs), Some(true)) {
            self.in_realtime_main_fn.push(hir_id);
        }
    }

    /// Parse closure markers at the statement level (let bindings)
    fn check_stmt(&mut self, cx: &LateContext<'tcx>, stmt: &'tcx hir::Stmt<'tcx>) {
        if let hir::StmtKind::Let(local) = stmt.kind {
            let attrs = cx.tcx.hir().attrs(stmt.hir_id);
            if let Some((name_marker, is_rt)) = Self::extract_call_info_from_doc_attrs(attrs) {
                if name_marker == "closure" {
                    if let hir::PatKind::Binding(_, _hir_id, ident, _pat) = local.pat.kind {
                        let var_name = ident.name.to_string();
                        self.closure_var_realtime.insert(var_name.clone(), is_rt);
                        eprintln_ts!(
                            "[dylint] record closure var '{}' as realtime={}",
                            var_name,
                            is_rt
                        );
                    }
                }
                // } else {
                //     // Treat as a function-pointer variable marker. Try to parse function DefId
                //     // from the RHS and decide realtime based on doc attributes.
                //     if let Some(init) = local.init {
                //         if let hir::ExprKind::Path(qpath) = init.kind {
                //             let res = cx.qpath_res(&qpath, init.hir_id);
                //             if let Some(def_id) = res.opt_def_id() {
                //                 let is_fn_rt = Self::callee_is_realtime(cx, def_id);
                //                 if let hir::PatKind::Binding(_, _hid, ident, _pat) = local.pat.kind
                //                 {
                //                     let var_name = ident.name.to_string();
                //                     self.fnptr_var_realtime.insert(var_name.clone(), is_fn_rt);
                //                     eprintln!(
                //                         "[dylint] record fn-ptr var '{}' -> target '{}' realtime={:?} (marker name='{}')",
                //                         var_name,
                //                         cx.tcx.def_path_str(def_id),
                //                         is_fn_rt,
                //                         name_marker,
                //                     );
                //                 }
                //             }
                //         }
                //     }
                // }
            }

            // Extra: without markers, directly detect function-pointer assignments in let (fn item/associated fn)
            if let Some(init) = local.init {
                if let hir::ExprKind::Path(qpath) = init.kind {
                    let res = cx.qpath_res(&qpath, init.hir_id);
                    if let Some(def_id) = res.opt_def_id() {
                        let kind = cx.tcx.def_kind(def_id);
                        if matches!(kind, DefKind::Fn | DefKind::AssocFn) {
                            if let hir::PatKind::Binding(_, _hid, ident, _pat) = local.pat.kind {
                                let var_name = ident.name.to_string();
                                let is_fn_rt = Self::callee_is_realtime(cx, def_id);
                                if let Some(is_fn_rt) = is_fn_rt {
                                    self.fnptr_var_realtime.insert(var_name.clone(), is_fn_rt);
                                }
                                eprintln_ts!(
                                    "[dylint] auto-detect fn-ptr var '{}' -> target '{}' realtime={:?}",
                                    var_name,
                                    cx.tcx.def_path_str(def_id),
                                    is_fn_rt,
                                );
                            }
                        }
                    }
                }
            }

            // Propagation: let new = old; make new inherit old's realtime property (closure/fn-ptr)
            if let Some(init) = local.init {
                if let hir::ExprKind::Path(qpath) = init.kind {
                    if let Res::Local(src_id) = cx.qpath_res(&qpath, init.hir_id) {
                        if let Some(sym) = cx.tcx.hir().opt_name(src_id) {
                            let src = sym.to_string();
                            if let hir::PatKind::Binding(_, _hid, ident, _pat) = local.pat.kind {
                                let dst = ident.name.to_string();
                                if let Some(v) = self.closure_var_realtime.get(&src).copied() {
                                    self.closure_var_realtime.insert(dst.clone(), v);
                                    eprintln_ts!(
                                        "[dylint] propagate closure realtime {} -> {} = {:?}",
                                        src,
                                        dst,
                                        v
                                    );
                                }
                                if let Some(v) = self.fnptr_var_realtime.get(&src).copied() {
                                    self.fnptr_var_realtime.insert(dst.clone(), v);
                                    eprintln_ts!(
                                        "[dylint] propagate fn-ptr realtime {} -> {} = {:?}",
                                        src,
                                        dst,
                                        v
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Check 'calls' inside the function body
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        // Only check inside realtime functions
        // eprintln!("[debug] check expr: {:?}", expr);
        eprintln_ts!("[debug] in realtime: {:?}", self.in_realtime());
        if !self.in_realtime() {
            return;
        }
        // eprintln!("[debug] in realtime");

        // 1) Method call: foo.bar(...)
        //trait and method call
        if let hir::ExprKind::MethodCall(_seg, _recv, _args, _fn_span) = expr.kind {
            if let Some(def_id) = cx.typeck_results().type_dependent_def_id(expr.hir_id) {
                if Self::callee_is_realtime(cx, def_id) == Some(false) {
                    cx.span_lint(REALTIME_CALLS_NONREALTIME, expr.span, |diag| {
                        diag.primary_message(format!(
                            "{} realtime function calls a non-realtime method",
                            crate::timestamp_prefix()
                        ));
                    });
                }
            }
        }

        // 2) Ordinary function call: path_fn(...)
        if let hir::ExprKind::Call(callee, _args) = expr.kind {
            // Resolve callee's DefId (handle common path forms)
            if let hir::ExprKind::Path(qpath) = callee.kind {
                let res = cx.qpath_res(&qpath, callee.hir_id);
                // A. Local variable (closure variable)
                if let Res::Local(local_id) = res {
                    if let Some(sym) = cx.tcx.hir().opt_name(local_id) {
                        let var_name = sym.to_string();
                        eprintln_ts!(
                            "[debug] check closure var '{}' realtime={:?}",
                            var_name,
                            self.closure_var_realtime.get(&var_name).copied()
                        );
                        if let Some(is_rt) = self.closure_var_realtime.get(&var_name).copied() {
                            if is_rt == false {
                                eprintln_ts!(
                                    "[dylint] nonrealtime closure call detected: {}",
                                    var_name
                                );
                                cx.span_lint(REALTIME_CALLS_NONREALTIME, expr.span, |diag| {
                                    diag.primary_message(format!(
                                        "{} Nonrealtime Closure call detected: {}",
                                        crate::timestamp_prefix(),
                                        var_name
                                    ));
                                });
                            }
                        }
                        if let Some(is_rt) = self.fnptr_var_realtime.get(&var_name).copied() {
                            if is_rt == false {
                                eprintln_ts!(
                                    "[dylint] nonrealtime fn-ptr call detected: {}",
                                    var_name
                                );
                                cx.span_lint(REALTIME_CALLS_NONREALTIME, expr.span, |diag| {
                                    diag.primary_message(format!(
                                        "{} Nonrealtime Fn-ptr call detected: {}",
                                        crate::timestamp_prefix(),
                                        var_name
                                    ));
                                });
                            }
                        }
                    }
                }
                // B. Ordinary function DefId
                if let Some(def_id) = res.opt_def_id() {
                    if Self::callee_is_realtime(cx, def_id) == Some(false) {
                        cx.span_lint(REALTIME_CALLS_NONREALTIME, expr.span, |diag| {
                            diag.primary_message(format!(
                                "{} nonrealtime function call detected: {}",
                                crate::timestamp_prefix(),
                                cx.tcx.def_path_str(def_id)
                            ));
                        });
                    }
                }
            }
        }
    }
}
/// This is the standard Dylint UI test setup
/// It looks for test files under `tests/ui` and verifies that the lint output matches expectations
#[test]
fn ui() {
    dylint_testing::ui_test(env!("CARGO_PKG_NAME"), "ui");
}
