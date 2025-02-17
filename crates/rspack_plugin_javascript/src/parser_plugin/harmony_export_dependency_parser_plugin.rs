use rspack_core::{ConstDependency, DependencyLocation, DependencyType, SpanExt};
use swc_core::atoms::Atom;
use swc_core::common::{Span, Spanned};
use swc_core::ecma::ast::{
  ClassDecl, Decl, DefaultDecl, ExportAll, ExportDefaultDecl, ExportSpecifier, FnDecl, Ident,
  ModuleExportName, NamedExport,
};
use swc_core::ecma::utils::{find_pat_ids, ExprFactory};

use super::harmony_import_dependency_parser_plugin::handle_harmony_import_side_effects_dep;
use super::{
  InnerGraphMapUsage, InnerGraphPlugin, JavascriptParserPlugin, DEFAULT_STAR_JS_WORD,
  JS_DEFAULT_KEYWORD,
};
use crate::dependency::{
  DeclarationId, DeclarationInfo, HarmonyExportExpressionDependency, HarmonyExportHeaderDependency,
  HarmonyExportImportedSpecifierDependency, HarmonyExportSpecifierDependency, Specifier,
};
use crate::visitors::JavascriptParser;

fn handle_esm_export_harmony_import_side_effects_dep(
  parser: &mut JavascriptParser,
  request: Atom,
  span: Span,
  source_span: Span,
  specifiers: Vec<Specifier>,
  dep_type: DependencyType,
  export_all: bool,
) {
  assert!(matches!(dep_type, DependencyType::EsmExport));
  if !specifiers.is_empty() {
    specifiers.iter().for_each(|specifier| match specifier {
      Specifier::Namespace(n) => {
        let ids = vec![(n.clone(), None)];
        parser
          .dependencies
          .push(Box::new(HarmonyExportImportedSpecifierDependency::new(
            request.clone(),
            parser.last_harmony_import_order,
            ids,
            vec![],
            Some(n.clone()),
            false,
            None,
            span.into(),
            HarmonyExportImportedSpecifierDependency::create_export_presence_mode(
              parser.javascript_options,
            ),
          )));
        parser.build_info.harmony_named_exports.insert(n.clone());
      }
      Specifier::Default(_) => {
        unreachable!()
      }
      Specifier::Named(orig, exported) => {
        let name = exported.clone().unwrap_or(orig.clone());
        let ids = vec![(name.clone(), Some(orig.clone()))];
        // TODO: add variable usage
        parser
          .dependencies
          .push(Box::new(HarmonyExportImportedSpecifierDependency::new(
            request.clone(),
            parser.last_harmony_import_order,
            ids.clone(),
            ids,
            Some(name.clone()),
            false,
            None,
            span.into(),
            HarmonyExportImportedSpecifierDependency::create_export_presence_mode(
              parser.javascript_options,
            ),
          )));
        parser.build_info.harmony_named_exports.insert(name);
      }
    });
  }

  handle_harmony_import_side_effects_dep(parser, request, span, source_span, dep_type, export_all)
}
pub struct HarmonyExportDependencyParserPlugin;

impl JavascriptParserPlugin for HarmonyExportDependencyParserPlugin {
  fn named_export_import(
    &self,
    parser: &mut JavascriptParser,
    named_export: &NamedExport,
    _source: &str,
  ) -> Option<bool> {
    let Some(src) = &named_export.src else {
      unreachable!()
    };
    parser.last_harmony_import_order += 1;
    let mut specifiers = vec![];
    named_export
      .specifiers
      .iter()
      .for_each(|specifier| match specifier {
        ExportSpecifier::Namespace(n) if let ModuleExportName::Ident(export) = &n.name => {
          specifiers.push(Specifier::Namespace(export.sym.clone()));
        }
        ExportSpecifier::Named(named) if let ModuleExportName::Ident(orig) = &named.orig => {
          specifiers.push(Specifier::Named(
            orig.sym.clone(),
            match &named.exported {
              Some(ModuleExportName::Str(export)) => Some(export.value.clone()),
              Some(ModuleExportName::Ident(export)) => Some(export.sym.clone()),
              None => None,
            },
          ));
        }
        ExportSpecifier::Default(_) => {
          // export a from "./a"; is a syntax error
          unreachable!()
        }
        _ => {}
      });

    handle_esm_export_harmony_import_side_effects_dep(
      parser,
      src.value.clone(),
      named_export.span,
      src.span,
      specifiers,
      DependencyType::EsmExport,
      false,
    );

    parser
      .presentational_dependencies
      .push(Box::new(ConstDependency::new(
        named_export.span.real_lo(),
        named_export.span.real_hi(),
        "".into(),
        None,
      )));
    Some(true)
  }

  fn all_export_import(
    &self,
    parser: &mut JavascriptParser,
    export_all: &ExportAll,
    _source: &str,
  ) -> Option<bool> {
    parser.last_harmony_import_order += 1;

    handle_esm_export_harmony_import_side_effects_dep(
      parser,
      export_all.src.value.clone(),
      export_all.span,
      export_all.src.span,
      vec![],
      DependencyType::EsmExport,
      true,
    );

    let list = Some(parser.build_info.all_star_exports.clone());
    let export_imported_dep = HarmonyExportImportedSpecifierDependency::new(
      export_all.src.value.clone(),
      parser.last_harmony_import_order,
      vec![],
      vec![],
      None,
      true,
      list,
      export_all.span.into(),
      HarmonyExportImportedSpecifierDependency::create_export_presence_mode(
        parser.javascript_options,
      ),
    );

    parser
      .build_info
      .all_star_exports
      .push(export_imported_dep.id);
    parser.dependencies.push(Box::new(export_imported_dep));

    parser
      .presentational_dependencies
      .push(Box::new(ConstDependency::new(
        export_all.span.real_lo(),
        export_all.span.real_hi(),
        "".into(),
        None,
      )));
    Some(true)
  }

  fn export(
    &self,
    parser: &mut JavascriptParser,
    export_default_decl: &ExportDefaultDecl,
  ) -> Option<bool> {
    let named_id = match &export_default_decl.decl {
      DefaultDecl::Class(class_expr) => class_expr.to_owned().as_class_decl().map(|c| c.ident.sym),
      DefaultDecl::Fn(fn_expr) => fn_expr.to_owned().as_fn_decl().map(|c| c.ident.sym),
      _ => None,
    };

    if let Some(named_id) = named_id
      && !named_id.is_empty()
    {
      parser
        .dependencies
        .push(Box::new(HarmonyExportSpecifierDependency::new(
          JS_DEFAULT_KEYWORD.clone(),
          named_id.clone(),
        )));
      InnerGraphPlugin::add_variable_usage(
        parser,
        &named_id,
        InnerGraphMapUsage::Value(JS_DEFAULT_KEYWORD.clone()),
      );
      parser
        .presentational_dependencies
        .push(Box::new(HarmonyExportHeaderDependency::new(
          Some(DependencyLocation::new(
            export_default_decl.decl.span().real_lo(),
            export_default_decl.decl.span().real_hi(),
            Some(parser.source_map.clone()),
          )),
          DependencyLocation::new(
            export_default_decl.span().real_lo(),
            export_default_decl.span().real_hi(),
            Some(parser.source_map.clone()),
          ),
        )));
      return Some(true);
    }

    let ident = match &export_default_decl.decl {
      DefaultDecl::Class(class_expr) => &class_expr.ident,
      DefaultDecl::Fn(f) => &f.ident,
      _ => unreachable!(),
    };
    let local = match &ident {
      Some(ident) => ident.sym.clone(),
      None => DEFAULT_STAR_JS_WORD.clone(),
    };
    InnerGraphPlugin::add_variable_usage(
      parser,
      &local,
      InnerGraphMapUsage::Value(JS_DEFAULT_KEYWORD.clone()),
    );

    parser
      .presentational_dependencies
      .push(Box::new(HarmonyExportExpressionDependency::new(
        DependencyLocation::new(
          export_default_decl.decl.span().real_lo(),
          export_default_decl.decl.span().real_hi(),
          Some(parser.source_map.clone()),
        ),
        DependencyLocation::new(
          export_default_decl.span().real_lo(),
          export_default_decl.span().real_hi(),
          Some(parser.source_map.clone()),
        ),
        match &export_default_decl.decl {
          DefaultDecl::Class(class_expr) => class_expr
            .ident
            .clone()
            .map(|i| DeclarationId::Id(i.sym.to_string())),
          DefaultDecl::Fn(f) => {
            let start = f.span().real_lo();
            let end = if let Some(first_arg) = f.function.params.first() {
              first_arg.span().real_lo()
            } else {
              f.function.body.span().real_lo()
            };
            Some(DeclarationId::Func(DeclarationInfo {
              range: DependencyLocation::new(start, end, Some(parser.source_map.clone())),
              prefix: format!(
                "{}function{} ",
                if f.function.is_async { "async " } else { "" },
                if f.function.is_generator { "*" } else { "" },
              ),
              suffix: format!(
                r#"({}"#,
                if f.function.params.is_empty() {
                  ") "
                } else {
                  ""
                }
              ),
            }))
          }
          _ => unreachable!(),
        },
      )));

    Some(true)
  }

  fn export_default_expr(
    &self,
    parser: &mut JavascriptParser,
    export_default_expr: &swc_core::ecma::ast::ExportDefaultExpr,
  ) -> Option<bool> {
    InnerGraphPlugin::add_variable_usage(
      parser,
      &DEFAULT_STAR_JS_WORD,
      InnerGraphMapUsage::Value(JS_DEFAULT_KEYWORD.clone()),
    );
    parser
      .presentational_dependencies
      .push(Box::new(HarmonyExportExpressionDependency::new(
        DependencyLocation::new(
          export_default_expr.expr.span().real_lo(),
          export_default_expr.expr.span().real_hi(),
          Some(parser.source_map.clone()),
        ),
        DependencyLocation::new(
          export_default_expr.span().real_lo(),
          export_default_expr.span().real_hi(),
          Some(parser.source_map.clone()),
        ),
        None,
      )));
    Some(true)
  }

  fn export_decl(
    &self,
    parser: &mut JavascriptParser,
    export_decl: &swc_core::ecma::ast::ExportDecl,
  ) -> Option<bool> {
    match &export_decl.decl {
      Decl::Class(ClassDecl { ident, .. }) | Decl::Fn(FnDecl { ident, .. }) => {
        parser
          .dependencies
          .push(Box::new(HarmonyExportSpecifierDependency::new(
            ident.sym.clone(),
            ident.sym.clone(),
          )));

        InnerGraphPlugin::add_variable_usage(
          parser,
          &ident.sym,
          InnerGraphMapUsage::Value(ident.sym.clone()),
        );
        parser
          .build_info
          .harmony_named_exports
          .insert(ident.sym.clone());
      }
      Decl::Var(v) => {
        find_pat_ids::<_, Ident>(&v.decls)
          .into_iter()
          .for_each(|ident| {
            parser
              .dependencies
              .push(Box::new(HarmonyExportSpecifierDependency::new(
                ident.sym.clone(),
                ident.sym.clone(),
              )));

            InnerGraphPlugin::add_variable_usage(
              parser,
              &ident.sym,
              InnerGraphMapUsage::Value(ident.sym.clone()),
            );
            parser.build_info.harmony_named_exports.insert(ident.sym);
          });
      }
      _ => {}
    }
    parser
      .presentational_dependencies
      .push(Box::new(HarmonyExportHeaderDependency::new(
        Some(DependencyLocation::new(
          export_decl.decl.span().real_lo(),
          export_decl.decl.span().real_hi(),
          Some(parser.source_map.clone()),
        )),
        DependencyLocation::new(
          export_decl.span().real_lo(),
          export_decl.span().real_hi(),
          Some(parser.source_map.clone()),
        ),
      )));
    Some(true)
  }

  fn named_export(
    &self,
    parser: &mut JavascriptParser,
    named_export: &NamedExport,
  ) -> Option<bool> {
    if named_export.src.is_none() {
      named_export
        .specifiers
        .iter()
        .for_each(|specifier| match specifier {
          ExportSpecifier::Named(named) => {
            if let ModuleExportName::Ident(orig) = &named.orig {
              let export = match &named.exported {
                Some(ModuleExportName::Ident(export)) => export.sym.clone(),
                Some(ModuleExportName::Str(name)) => name.value.clone(),
                None => orig.sym.clone(),
              };
              if let Some(reference) = parser.import_map.get(&orig.to_id()) {
                let ids = vec![(export.clone(), reference.names.clone())];
                // dbg!(&reference);
                let mode_ids = match reference.specifier {
                  Specifier::Namespace(_) => {
                    vec![]
                  }
                  _ => ids.clone(),
                };
                parser
                  .dependencies
                  .push(Box::new(HarmonyExportImportedSpecifierDependency::new(
                    reference.request.clone(),
                    reference.source_order,
                    ids,
                    mode_ids,
                    Some(export.clone()),
                    false,
                    None,
                    named.span.into(),
                    HarmonyExportImportedSpecifierDependency::create_export_presence_mode(
                      parser.javascript_options,
                    ),
                  )));
                parser
                  .build_info
                  .harmony_named_exports
                  .insert(export.clone());
              } else {
                parser
                  .dependencies
                  .push(Box::new(HarmonyExportSpecifierDependency::new(
                    export.clone(),
                    orig.sym.clone(),
                  )));

                parser
                  .build_info
                  .harmony_named_exports
                  .insert(export.clone());
              }
              InnerGraphPlugin::add_variable_usage(
                parser,
                &orig.sym,
                InnerGraphMapUsage::Value(export),
              );
            }
          }
          _ => unreachable!(),
        });

      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          named_export.span.real_lo(),
          named_export.span.real_hi(),
          "".into(),
          None,
        )));
      Some(true)
    } else {
      None
    }
  }
}
