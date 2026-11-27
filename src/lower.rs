use swc_core::common::comments::SingleThreadedComments;
use swc_core::common::sync::Lrc;
use swc_core::common::{FileName, GLOBALS, Globals, Mark, SourceMap, Span, Spanned};
use swc_core::ecma::ast::{EsVersion, Program};
use swc_core::ecma::codegen::{Config, Emitter, text_writer::JsWriter};
use swc_core::ecma::parser::{EsSyntax, Parser, StringInput, Syntax};
use swc_core::ecma::preset_env::transform_from_es_version;
use swc_core::ecma::transforms::base::helpers::{HELPERS, Helpers, inject_helpers};
use swc_core::ecma::transforms::base::{
    assumptions::Assumptions, fixer::fixer, hygiene::hygiene, resolver,
};
use swc_core::ecma::transforms::module::common_js::{self, FeatureFlag};
use swc_core::ecma::transforms::module::path::Resolver;
use swc_core::ecma::transforms::module::util::ImportInterop;

use crate::bundle::{Bundle, Data};
use crate::commonjs;

pub struct Options {
    pub target: EsVersion,
    pub format: Format,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Format {
    Esm,
    Cjs,
}

pub struct Diagnostic {
    pub url: String,
    pub start: u32,
    pub end: u32,
    pub message: String,
}

impl Diagnostic {
    pub fn new(url: &str, message: impl Into<String>) -> Self {
        Diagnostic {
            url: url.into(),
            start: 0,
            end: 0,
            message: message.into(),
        }
    }
}

pub fn parse_target(target: &str) -> Option<EsVersion> {
    Some(match target {
        "es5" => EsVersion::Es5,
        "es2015" => EsVersion::Es2015,
        "es2016" => EsVersion::Es2016,
        "es2017" => EsVersion::Es2017,
        "es2018" => EsVersion::Es2018,
        "es2019" => EsVersion::Es2019,
        "es2020" => EsVersion::Es2020,
        "es2021" => EsVersion::Es2021,
        "es2022" => EsVersion::Es2022,
        "es2023" => EsVersion::Es2023,
        "es2024" => EsVersion::Es2024,
        "esnext" => EsVersion::EsNext,
        _ => return None,
    })
}

pub fn parse_format(format: &str) -> Option<Format> {
    match format {
        "esm" => Some(Format::Esm),
        "cjs" => Some(Format::Cjs),
        _ => None,
    }
}

pub fn lower(input: &[u8], options: &Options) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let mut bundle = Bundle::parse(input).map_err(|message| vec![Diagnostic::new("", message)])?;

    let assets: Vec<String> = bundle.assets().map(String::from).collect();

    let mut requires = Vec::new();
    let mut diagnostics = Vec::new();

    for file in &mut bundle.files {
        if assets.contains(&file.url) {
            continue;
        }

        let Some(kind) = kind(file.kind.as_deref(), &file.url) else {
            continue;
        };

        let Ok(source) = std::str::from_utf8(file.data.as_bytes()) else {
            diagnostics.push(Diagnostic::new(&file.url, "Module is not valid UTF-8"));
            continue;
        };

        match lower_module(source, kind, options) {
            Ok(lowered) => {
                // A file without a recorded type might still load as an ES
                // module, such as through the package "type" field.
                if options.format == Format::Cjs {
                    file.kind = Some("script".into());
                }

                if lowered.converted {
                    requires.push((file.url.clone(), None));
                } else if !lowered.dynamic_imports.is_empty() {
                    requires.push((file.url.clone(), Some(lowered.dynamic_imports)));
                }

                file.data = Data::Owned(lowered.code.into_bytes());
            }
            Err(errors) => {
                diagnostics.extend(errors.into_iter().map(|(start, end, message)| Diagnostic {
                    url: file.url.clone(),
                    start,
                    end,
                    message,
                }))
            }
        }
    }

    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    // A module that keeps its own `require()` calls shares their resolutions
    // with its lowered imports, which can't tell them apart once lowered.
    for (url, imports) in &requires {
        for (specifier, start, end) in imports.iter().flatten() {
            if bundle.resolves_differently_as_require(url, specifier) {
                diagnostics.push(Diagnostic {
                    url: url.clone(),
                    start: *start,
                    end: *end,
                    message: format!(
                        "Cannot lower import of '{specifier}' as it resolves to a different module for require()"
                    ),
                });
            }
        }
    }

    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    for (url, imports) in requires {
        let specifiers: Option<Vec<String>> = imports.map(|imports| {
            imports
                .into_iter()
                .map(|(specifier, _, _)| specifier)
                .collect()
        });

        bundle.resolve_imports_as_requires(&url, specifiers.as_deref());
    }

    Ok(bundle.serialize())
}

#[derive(Clone, Copy)]
enum Kind {
    Script,
    Module,
    Unambiguous,
}

fn kind(kind: Option<&str>, url: &str) -> Option<Kind> {
    match kind {
        Some("script") => Some(Kind::Script),
        Some("module") => Some(Kind::Module),
        Some(_) => None,
        None => kind_for_extension(url),
    }
}

// Without a recorded type, a '.js' file may be either CommonJS or an ES module,
// so let the parser decide from the presence of module syntax.
fn kind_for_extension(url: &str) -> Option<Kind> {
    if url.ends_with(".cjs") {
        Some(Kind::Script)
    } else if url.ends_with(".mjs") {
        Some(Kind::Module)
    } else if url.ends_with(".js") {
        Some(Kind::Unambiguous)
    } else {
        None
    }
}

struct Lowered {
    code: String,
    converted: bool,
    dynamic_imports: Vec<(String, u32, u32)>,
}

type Errors = Vec<(u32, u32, String)>;

fn lower_module(source: &str, kind: Kind, options: &Options) -> Result<Lowered, Errors> {
    let Options { target, format } = *options;

    let cm: Lrc<SourceMap> = Default::default();
    let file = cm.new_source_file(FileName::Anon.into(), source.to_owned());
    let comments = SingleThreadedComments::default();

    let offset = |span: Span| {
        (
            span.lo.0.saturating_sub(file.start_pos.0),
            span.hi.0.saturating_sub(file.start_pos.0),
        )
    };

    let errors = |errors: Vec<(Span, String)>| -> Errors {
        errors
            .into_iter()
            .map(|(span, message)| {
                let (start, end) = offset(span);
                (start, end, message)
            })
            .collect()
    };

    let syntax = Syntax::Es(EsSyntax {
        import_attributes: true,
        explicit_resource_management: true,
        // Bare evaluates every module that is not an ES module inside a
        // function wrapper, where returning from the top level is allowed.
        allow_return_outside_function: !matches!(kind, Kind::Module),
        ..EsSyntax::default()
    });

    let mut parser = Parser::new(syntax, StringInput::from(&*file), Some(&comments));

    let parsed = match kind {
        Kind::Script => parser.parse_script().map(Program::Script),
        Kind::Module => parser.parse_module().map(Program::Module),
        Kind::Unambiguous => parser.parse_program(),
    };

    let mut parse_errors: Vec<(Span, String)> = parser
        .take_errors()
        .into_iter()
        .map(|error| (error.span(), error.kind().msg().into_owned()))
        .collect();

    let mut program = match parsed {
        Ok(program) if parse_errors.is_empty() => program,
        Ok(_) => return Err(errors(parse_errors)),
        Err(error) => {
            parse_errors.push((error.span(), error.kind().msg().into_owned()));
            return Err(errors(parse_errors));
        }
    };

    let convert = program.is_module() && format == Format::Cjs;

    let dynamic_imports = GLOBALS.set(&Globals::new(), || {
        HELPERS.set(&Helpers::new(false), || {
            let unresolved_mark = Mark::new();
            let top_level_mark = Mark::new();

            program.mutate(resolver(unresolved_mark, top_level_mark, false));

            let mut attributes = commonjs::ImportAttributes::new();
            let mut dynamic_imports = Vec::new();

            if convert {
                let top_level_await = commonjs::check_top_level_await(&program);

                if !top_level_await.is_empty() {
                    return Err(errors(top_level_await));
                }

                commonjs::rewrite_import_meta(&mut program, unresolved_mark);

                attributes = commonjs::collect_import_attributes(&program);
            }

            if format == Format::Cjs {
                dynamic_imports = commonjs::rewrite_dynamic_imports(&mut program, unresolved_mark);
            }

            if target != EsVersion::EsNext {
                program.mutate(transform_from_es_version(
                    unresolved_mark,
                    Some(&comments),
                    target,
                    Assumptions::default(),
                    false,
                ));
            }

            if convert {
                program.mutate(common_js::common_js(
                    Resolver::Default,
                    unresolved_mark,
                    common_js::Config {
                        import_interop: Some(ImportInterop::Swc),
                        ignore_dynamic: true,
                        ..Default::default()
                    },
                    FeatureFlag::default(),
                ));

                commonjs::restore_import_attributes(&mut program, &attributes, unresolved_mark);
            }

            program.mutate(inject_helpers(unresolved_mark));
            program.mutate(hygiene());
            program.mutate(fixer(Some(&comments)));

            Ok(dynamic_imports)
        })
    })?;

    let mut code = Vec::new();

    let mut emitter = Emitter {
        cfg: Config::default().with_target(target),
        cm: cm.clone(),
        comments: Some(&comments),
        wr: JsWriter::new(cm.clone(), "\n", &mut code, None),
    };

    emitter
        .emit_program(&program)
        .map_err(|error| vec![(0, 0, error.to_string())])?;

    Ok(Lowered {
        code: String::from_utf8(code).expect("Codegen produced invalid UTF-8"),
        converted: convert,
        dynamic_imports: dynamic_imports
            .into_iter()
            .map(|(specifier, span)| {
                let (start, end) = offset(span);
                (specifier, start, end)
            })
            .collect(),
    })
}
