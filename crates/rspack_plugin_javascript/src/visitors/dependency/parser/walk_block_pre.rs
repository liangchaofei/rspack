use swc_core::common::Spanned;
use swc_core::ecma::ast::{
  ClassDecl, ClassExpr, ExportDecl, Expr, FnDecl, ImportDecl, ImportSpecifier, ModuleExportName,
};
use swc_core::ecma::ast::{Decl, DefaultDecl, ExportAll, ExportDefaultDecl, ExprStmt};
use swc_core::ecma::ast::{ModuleDecl, ModuleItem, NamedExport, Stmt, VarDecl, VarDeclKind};

use super::JavascriptParser;
use crate::parser_plugin::JavascriptParserPlugin;

impl<'parser> JavascriptParser<'parser> {
  pub fn block_pre_walk_module_declarations(&mut self, statements: &Vec<ModuleItem>) {
    for statement in statements {
      self.block_pre_walk_module_declaration(statement);
    }
  }

  pub fn block_pre_walk_statements(&mut self, statements: &Vec<Stmt>) {
    for statement in statements {
      self.block_pre_walk_statement(statement);
    }
  }

  pub fn block_pre_walk_module_declaration(&mut self, statement: &ModuleItem) {
    match statement {
      ModuleItem::ModuleDecl(decl) => {
        self.statement_path.push(decl.span().into());
        // TODO: `hooks.block_pre_statement.call`
        match decl {
          ModuleDecl::Import(decl) => self.block_pre_walk_import_declaration(decl),
          ModuleDecl::ExportAll(decl) => self.block_pre_walk_export_all_declaration(decl),
          ModuleDecl::ExportDefaultDecl(decl) => {
            self.block_pre_walk_export_default_declaration(decl)
          }
          ModuleDecl::ExportNamed(decl) => self.block_pre_walk_export_name_declaration(decl),
          ModuleDecl::ExportDefaultExpr(default_expr) => {
            self.block_pre_walk_expression_statement(&ExprStmt {
              span: default_expr.span,
              expr: default_expr.expr.clone(),
            })
          }
          ModuleDecl::ExportDecl(exp) => self.block_pre_walk_export_declaration(exp),
          ModuleDecl::TsImportEquals(_)
          | ModuleDecl::TsExportAssignment(_)
          | ModuleDecl::TsNamespaceExport(_) => unreachable!(),
        };
        self.prev_statement = self.statement_path.pop();
      }
      ModuleItem::Stmt(stmt) => self.block_pre_walk_statement(stmt),
    }
  }

  pub fn block_pre_walk_statement(&mut self, stmt: &Stmt) {
    self.statement_path.push(stmt.span().into());
    if self
      .plugin_drive
      .clone()
      .pre_block_statement(self, stmt)
      .unwrap_or_default()
    {
      self.prev_statement = self.statement_path.pop();
      return;
    }

    match stmt {
      Stmt::Decl(stmt) => match stmt {
        Decl::Class(decl) => self.block_pre_walk_class_declaration(decl),
        Decl::Var(decl) => self.block_pre_walk_variable_declaration(decl),
        Decl::Fn(_) | Decl::Using(_) => (),
        Decl::TsInterface(_) | Decl::TsTypeAlias(_) | Decl::TsEnum(_) | Decl::TsModule(_) => {
          unreachable!()
        }
      },
      Stmt::Expr(expr) => self.block_pre_walk_expression_statement(expr),
      _ => (),
    }
    self.prev_statement = self.statement_path.pop();
  }

  fn block_pre_walk_expression_statement(&mut self, stmt: &ExprStmt) {
    if let Some(assign) = stmt.expr.as_assign() {
      self.pre_walk_assignment_expression(assign)
    }
  }

  pub(super) fn block_pre_walk_variable_declaration(&mut self, decl: &VarDecl) {
    if decl.kind != VarDeclKind::Var {
      self._pre_walk_variable_declaration(decl);
    }
  }

  fn block_pre_walk_export_name_declaration(&mut self, decl: &NamedExport) {
    if let Some(source) = &decl.src {
      self
        .plugin_drive
        .clone()
        .named_export_import(self, decl, source.value.as_str());
    } else {
      // TODO: `hooks.export.call`
    }
  }

  fn block_pre_walk_export_declaration(&mut self, exp: &ExportDecl) {
    // todo: move `hooks.export_decl.call` here
    self.pre_walk_export_declaration(exp);
  }

  fn block_pre_walk_class_declaration(&mut self, decl: &ClassDecl) {
    self.define_variable(decl.ident.sym.to_string())
  }

  fn block_pre_walk_export_default_declaration(&mut self, decl: &ExportDefaultDecl) {
    // FIXME: webpack use `self.pre_walk_statement(decl.decl)`
    match &decl.decl {
      DefaultDecl::Class(expr) => {
        if let Some(ident) = &expr.ident {
          self.define_variable(ident.sym.to_string());
          self.pre_walk_statement(&Stmt::Decl(Decl::Class(ClassDecl {
            ident: ident.clone(),
            declare: false,
            class: expr.class.clone(),
          })));
          self.block_pre_walk_statement(&Stmt::Decl(Decl::Class(ClassDecl {
            ident: ident.clone(),
            declare: false,
            class: expr.class.clone(),
          })));
        } else {
          self.pre_walk_statement(&Stmt::Expr(ExprStmt {
            span: expr.span(),
            expr: Box::new(Expr::Class(ClassExpr {
              ident: None,
              class: expr.class.clone(),
            })),
          }));
          self.block_pre_walk_statement(&Stmt::Expr(ExprStmt {
            span: expr.span(),
            expr: Box::new(Expr::Class(ClassExpr {
              ident: None,
              class: expr.class.clone(),
            })),
          }));
        }
      }
      DefaultDecl::Fn(expr) => {
        if let Some(ident) = &expr.ident {
          self.define_variable(ident.sym.to_string());
          self.pre_walk_statement(&Stmt::Decl(Decl::Fn(FnDecl {
            ident: ident.clone(),
            declare: false,
            function: expr.function.clone(),
          })));
          self.block_pre_walk_statement(&Stmt::Decl(Decl::Fn(FnDecl {
            ident: ident.clone(),
            declare: false,
            function: expr.function.clone(),
          })));
        } else {
          self.pre_walk_statement(&Stmt::Expr(ExprStmt {
            span: expr.span(),
            expr: Box::new(Expr::Fn(expr.clone())),
          }));
          self.block_pre_walk_statement(&Stmt::Expr(ExprStmt {
            span: expr.span(),
            expr: Box::new(Expr::Fn(expr.clone())),
          }));
        }
      }
      DefaultDecl::TsInterfaceDecl(_) => unreachable!(),
    }

    // FIXME: webpack use `self.block_pre_walk_statement(decl.decl)`
    // match &decl.decl {
    //   DefaultDecl::Class(expr) => {
    //     if let Some(ident) = &expr.ident {
    //       self.define_variable(ident.sym.to_string())
    //     }
    //   }
    //   DefaultDecl::Fn(expr) => {
    //     if let Some(ident) = &expr.ident {
    //       self.define_variable(ident.sym.to_string())
    //     }
    //   }
    //   DefaultDecl::TsInterfaceDecl(_) => unreachable!(),
    // }
  }

  fn block_pre_walk_export_all_declaration(&mut self, decl: &ExportAll) {
    self
      .plugin_drive
      .clone()
      .all_export_import(self, decl, decl.src.value.as_str());
    // TODO: `hooks.export_import_specifier.call`
  }

  fn block_pre_walk_import_declaration(&mut self, decl: &ImportDecl) {
    let drive = self.plugin_drive.clone();
    let source = &decl.src.value;
    drive.import(self, decl, source.as_str());

    for specifier in &decl.specifiers {
      match specifier {
        ImportSpecifier::Named(named) => {
          let identifier_name = &named.local.sym;
          let export_name = named
            .imported
            .as_ref()
            .map(|imported| match imported {
              ModuleExportName::Ident(ident) => &ident.sym,
              ModuleExportName::Str(s) => &s.value,
            })
            .unwrap_or_else(|| &named.local.sym);
          if drive
            .import_specifier(self, decl, source, Some(export_name), identifier_name)
            .unwrap_or_default()
          {
            self.define_variable(identifier_name.to_string())
          }
        }
        ImportSpecifier::Default(default) => {
          let identifier_name = &default.local.sym;
          if drive
            .import_specifier(self, decl, source, Some(&"default".into()), identifier_name)
            .unwrap_or_default()
          {
            self.define_variable(identifier_name.to_string())
          }
        }
        ImportSpecifier::Namespace(namespace) => {
          let identifier_name = &namespace.local.sym;
          if drive
            .import_specifier(self, decl, source, None, identifier_name)
            .unwrap_or_default()
          {
            self.define_variable(identifier_name.to_string())
          }
        }
      }
    }
  }
}
