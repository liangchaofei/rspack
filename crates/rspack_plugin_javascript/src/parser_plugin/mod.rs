mod api_plugin;
mod check_var_decl;
mod common_js_exports_parse_plugin;
mod common_js_imports_parse_plugin;
mod common_js_plugin;
mod compatibility_plugin;
mod r#const;
mod drive;
mod exports_info_api_plugin;
mod harmony_detection_parser_plugin;
mod harmony_export_dependency_parser_plugin;
mod harmony_import_dependency_parser_plugin;
mod harmony_top_level_this_plugin;
mod hot_module_replacement_plugin;
mod import_meta_context_dependency_parser_plugin;
mod import_meta_plugin;
mod import_parser_plugin;
mod initialize_evaluating;
mod inner_graph;
mod javascript_meta_info_plugin;
mod node_stuff_plugin;
mod require_context_dependency_parser_plugin;
mod r#trait;
mod url_plugin;
mod use_strict_plugin;
mod webpack_included_plugin;
mod worker_plugin;

pub mod define_plugin;
pub mod provide_plugin;

pub(crate) use self::api_plugin::APIPlugin;
pub(crate) use self::check_var_decl::CheckVarDeclaratorIdent;
pub(crate) use self::common_js_exports_parse_plugin::CommonJsExportsParserPlugin;
pub(crate) use self::common_js_imports_parse_plugin::CommonJsImportsParserPlugin;
pub(crate) use self::common_js_plugin::CommonJsPlugin;
pub(crate) use self::compatibility_plugin::CompatibilityPlugin;
pub(crate) use self::drive::JavaScriptParserPluginDrive;
pub(crate) use self::exports_info_api_plugin::ExportsInfoApiPlugin;
pub(crate) use self::harmony_detection_parser_plugin::HarmonyDetectionParserPlugin;
pub(crate) use self::harmony_export_dependency_parser_plugin::HarmonyExportDependencyParserPlugin;
pub(crate) use self::harmony_import_dependency_parser_plugin::HarmonyImportDependencyParserPlugin;
pub(crate) use self::harmony_top_level_this_plugin::HarmonyTopLevelThisParserPlugin;
pub(crate) use self::hot_module_replacement_plugin::hot_module_replacement;
pub(crate) use self::import_meta_context_dependency_parser_plugin::ImportMetaContextDependencyParserPlugin;
pub(crate) use self::import_meta_plugin::ImportMetaPlugin;
pub(crate) use self::import_parser_plugin::ImportParserPlugin;
pub(crate) use self::initialize_evaluating::InitializeEvaluating;
pub(crate) use self::inner_graph::{plugin::*, state::InnerGraphState};
pub(crate) use self::javascript_meta_info_plugin::JavascriptMetaInfoPlugin;
pub(crate) use self::node_stuff_plugin::NodeStuffPlugin;
pub(crate) use self::r#const::{is_logic_op, ConstPlugin};
pub use self::r#trait::{BoxJavascriptParserPlugin, JavascriptParserPlugin};
pub(crate) use self::require_context_dependency_parser_plugin::RequireContextDependencyParserPlugin;
pub(crate) use self::url_plugin::URLPlugin;
pub(crate) use self::use_strict_plugin::UseStrictPlugin;
pub(crate) use self::webpack_included_plugin::WebpackIsIncludedPlugin;
pub(crate) use self::worker_plugin::WorkerPlugin;

pub static JS_DEFAULT_KEYWORD: once_cell::sync::Lazy<swc_core::atoms::Atom> =
  once_cell::sync::Lazy::new(|| swc_core::atoms::atom!("default"));

pub static DEFAULT_STAR_JS_WORD: once_cell::sync::Lazy<swc_core::atoms::Atom> =
  once_cell::sync::Lazy::new(|| swc_core::atoms::atom!("*default*"));
