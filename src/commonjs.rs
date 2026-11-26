use std::collections::HashMap;

use swc_core::common::{DUMMY_SP, Mark, Span, SyntaxContext};
use swc_core::ecma::ast::*;
use swc_core::ecma::transforms::base::enable_helper;
use swc_core::ecma::utils::{ExprFactory, private_ident, quote_ident};
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

// Rewrite `import()` to `require()` in a promise and return the specifiers of
// the imports that are string literals.
pub fn rewrite_dynamic_imports(program: &mut Program, unresolved_mark: Mark) -> Vec<String> {
    let mut visitor = DynamicImports {
        ctxt: SyntaxContext::empty().apply_mark(unresolved_mark),
        specifiers: Vec::new(),
    };

    program.visit_mut_with(&mut visitor);

    visitor.specifiers
}

struct DynamicImports {
    ctxt: SyntaxContext,
    specifiers: Vec<String>,
}

impl VisitMut for DynamicImports {
    fn visit_mut_expr(&mut self, n: &mut Expr) {
        n.visit_mut_children_with(self);

        let Expr::Call(CallExpr {
            span,
            callee:
                Callee::Import(Import {
                    phase: ImportPhase::Evaluation,
                    ..
                }),
            args,
            ..
        }) = n
        else {
            return;
        };

        let mut args = std::mem::take(args);
        let specifier = args.remove(0);

        // A specifier that is not a literal is evaluated before the import
        // starts, as with `import()`, and passed on through the promise.
        let (resolved, params, specifier) = match &*specifier.expr {
            Expr::Lit(Lit::Str(literal)) => {
                self.specifiers
                    .push(literal.value.to_atom_lossy().to_string());

                (Vec::new(), Vec::new(), specifier)
            }
            _ => {
                let param = private_ident!("specifier");

                (vec![specifier], vec![param.clone().into()], param.as_arg())
            }
        };

        let interop = SyntaxContext::empty().apply_mark(enable_helper!(interop_require_wildcard));

        let require =
            quote_ident!(self.ctxt, "require").as_call(DUMMY_SP, [vec![specifier], args].concat());

        let then = quote_ident!(interop, "_interop_require_wildcard")
            .as_call(DUMMY_SP, vec![require.as_arg()])
            .into_lazy_arrow(params);

        *n = quote_ident!(self.ctxt, "Promise")
            .make_member(quote_ident!("resolve"))
            .as_call(DUMMY_SP, resolved)
            .make_member(quote_ident!("then"))
            .as_call(*span, vec![then.as_arg()]);
    }
}

pub type ImportAttributes = HashMap<Span, Box<ObjectLit>>;

// The attributes of the static imports and exports of `program`, by the span
// of their specifier.
pub fn collect_import_attributes(program: &Program) -> ImportAttributes {
    let Program::Module(module) = program else {
        return ImportAttributes::new();
    };

    module
        .body
        .iter()
        .filter_map(|item| match item.as_module_decl()? {
            ModuleDecl::Import(ImportDecl {
                src,
                with: Some(with),
                ..
            })
            | ModuleDecl::ExportAll(ExportAll {
                src,
                with: Some(with),
                ..
            })
            | ModuleDecl::ExportNamed(NamedExport {
                src: Some(src),
                with: Some(with),
                ..
            }) => Some((src.span, with.clone())),
            _ => None,
        })
        .collect()
}

// The conversion to CommonJS drops the attributes of static imports, so pass
// them on to the `require()` calls that load the same specifiers.
pub fn restore_import_attributes(
    program: &mut Program,
    attributes: &ImportAttributes,
    unresolved_mark: Mark,
) {
    if attributes.is_empty() {
        return;
    }

    program.visit_mut_with(&mut RestoreImportAttributes {
        ctxt: SyntaxContext::empty().apply_mark(unresolved_mark),
        attributes,
    });
}

struct RestoreImportAttributes<'a> {
    ctxt: SyntaxContext,
    attributes: &'a ImportAttributes,
}

impl VisitMut for RestoreImportAttributes<'_> {
    fn visit_mut_call_expr(&mut self, n: &mut CallExpr) {
        n.visit_mut_children_with(self);

        let Callee::Expr(callee) = &n.callee else {
            return;
        };

        let Expr::Ident(Ident { sym, ctxt, .. }) = &**callee else {
            return;
        };

        if &**sym != "require" || *ctxt != self.ctxt {
            return;
        }

        let [ExprOrSpread { spread: None, expr }] = &n.args[..] else {
            return;
        };

        let Expr::Lit(Lit::Str(specifier)) = &**expr else {
            return;
        };

        if let Some(with) = self.attributes.get(&specifier.span) {
            let options = ObjectLit {
                span: DUMMY_SP,
                props: vec![
                    Prop::KeyValue(KeyValueProp {
                        key: quote_ident!("with").into(),
                        value: Box::new(Expr::Object(*with.clone())),
                    })
                    .into(),
                ],
            };

            n.args.push(options.as_arg());
        }
    }
}
