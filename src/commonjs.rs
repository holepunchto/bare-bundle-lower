use swc_core::common::{Mark, Span, SyntaxContext};
use swc_core::ecma::ast::*;
use swc_core::ecma::utils::{ExprFactory, quote_ident};
use swc_core::ecma::visit::{Visit, VisitMut, VisitMutWith, VisitWith, noop_visit_type};

pub type Error = (Span, String);

pub fn check_top_level_await(program: &Program) -> Vec<Error> {
    let mut checker = TopLevelAwait::default();
    program.visit_with(&mut checker);
    checker.errors
}

#[derive(Default)]
struct TopLevelAwait {
    errors: Vec<Error>,
}

impl TopLevelAwait {
    fn report(&mut self, span: Span) {
        self.errors
            .push((span, "Top-level await cannot be lowered to CommonJS".into()));
    }
}

impl Visit for TopLevelAwait {
    noop_visit_type!();

    fn visit_function(&mut self, _: &Function) {}

    fn visit_arrow_expr(&mut self, _: &ArrowExpr) {}

    fn visit_await_expr(&mut self, n: &AwaitExpr) {
        self.report(n.span);
    }

    fn visit_for_of_stmt(&mut self, n: &ForOfStmt) {
        if n.is_await {
            self.report(n.span);
        }

        n.visit_children_with(self);
    }

    fn visit_using_decl(&mut self, n: &UsingDecl) {
        if n.is_await {
            self.report(n.span);
        }

        n.visit_children_with(self);
    }
}

const IMPORT_META_PROPERTIES: [&str; 8] = [
    "url", "main", "cache", "dirname", "filename", "resolve", "addon", "asset",
];

// Rewrite `import.meta` to the matching CommonJS bindings of Bare.
pub fn rewrite_import_meta(program: &mut Program, unresolved_mark: Mark) {
    program.visit_mut_with(&mut ImportMeta {
        ctxt: SyntaxContext::empty().apply_mark(unresolved_mark),
    });
}

struct ImportMeta {
    ctxt: SyntaxContext,
}

impl ImportMeta {
    fn global(&self, name: &str) -> Expr {
        quote_ident!(self.ctxt, name).into()
    }

    fn property(&self, name: &str, span: Span) -> Option<Expr> {
        Some(match name {
            "url" => self
                .global("module")
                .make_member(quote_ident!("url"))
                .make_member(quote_ident!("href"))
                .into(),
            "main" => BinExpr {
                span,
                op: op!("==="),
                left: self
                    .global("require")
                    .make_member(quote_ident!("main"))
                    .into(),
                right: self.global("module").into(),
            }
            .into(),
            "dirname" => self.global("__dirname"),
            "filename" => self.global("__filename"),
            "cache" | "resolve" | "addon" | "asset" => self
                .global("require")
                .make_member(quote_ident!(name))
                .into(),
            _ => return None,
        })
    }

    fn object(&self, span: Span) -> Expr {
        ObjectLit {
            span,
            props: IMPORT_META_PROPERTIES
                .iter()
                .map(|&name| {
                    Prop::KeyValue(KeyValueProp {
                        key: quote_ident!(name).into(),
                        value: self.property(name, span).unwrap().into(),
                    })
                    .into()
                })
                .collect(),
        }
        .into()
    }
}

impl VisitMut for ImportMeta {
    fn visit_mut_expr(&mut self, n: &mut Expr) {
        if let Some(property) =
            import_meta_property(n).and_then(|(span, name)| self.property(name, span))
        {
            *n = property;
        } else if let Expr::MetaProp(MetaPropExpr {
            span,
            kind: MetaPropKind::ImportMeta,
        }) = n
        {
            *n = self.object(*span);
        } else {
            n.visit_mut_children_with(self);
        }
    }
}

// The span and property name of `expr` if it is `import.meta.<name>`.
fn import_meta_property(expr: &Expr) -> Option<(Span, &str)> {
    match expr {
        Expr::Member(MemberExpr {
            obj,
            prop: MemberProp::Ident(prop),
            span,
        }) if matches!(
            &**obj,
            Expr::MetaProp(MetaPropExpr {
                kind: MetaPropKind::ImportMeta,
                ..
            })
        ) =>
        {
            Some((*span, &prop.sym))
        }
        _ => None,
    }
}
