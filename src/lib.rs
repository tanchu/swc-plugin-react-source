//! SWC plugin: adds `data-source="path:line"` to JSX elements (HTML and configured UI library components).
//! Equivalent to the Babel plugin `babel-plugin-react-source-string`.

use serde::Deserialize;
use std::collections::HashSet;
use swc_core::common::{SourceMapper, SyntaxContext, DUMMY_SP};
use swc_core::ecma::ast::*;
use swc_core::ecma::visit::{VisitMut, VisitMutWith};
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PluginConfig {
    libraries: Option<Vec<String>>,
    excluded: Option<Vec<String>>,
}

fn parse_config(metadata: &TransformPluginProgramMetadata) -> (HashSet<String>, HashSet<String>) {
    let config_str = match metadata.get_transform_plugin_config() {
        Some(s) => s,
        None => return (HashSet::new(), HashSet::new()),
    };
    let config: PluginConfig = match serde_json::from_str(&config_str) {
        Ok(c) => c,
        Err(_) => return (HashSet::new(), HashSet::new()),
    };
    let libraries = config
        .libraries
        .map(|v| v.into_iter().collect())
        .unwrap_or_default();
    let excluded = config
        .excluded
        .map(|v| v.into_iter().map(|s| s.to_lowercase()).collect())
        .unwrap_or_default();
    (libraries, excluded)
}

/// Returns path relative to cwd. Uses forward slashes (WASM path is unix-style).
fn relative_path(cwd: &str, filename: &str) -> String {
    let cwd = cwd.trim_end_matches('/');
    let filename = filename.replace('\\', "/");
    if let Some(stripped) = filename.strip_prefix(cwd) {
        stripped.trim_start_matches('/').to_string()
    } else {
        filename
    }
}

struct ReactSourceStringVisitor {
    libraries: HashSet<String>,
    excluded: HashSet<String>,
    ui_imports: HashSet<String>,
    source_map: swc_core::plugin::proxies::PluginSourceMapProxy,
    cwd: Option<String>,
}

impl ReactSourceStringVisitor {
    fn new(
        libraries: HashSet<String>,
        excluded: HashSet<String>,
        source_map: swc_core::plugin::proxies::PluginSourceMapProxy,
        metadata: &TransformPluginProgramMetadata,
    ) -> Self {
        let cwd = metadata
            .get_experimental_context("cwd")
            .filter(|s| !s.is_empty());
        Self {
            libraries,
            excluded,
            ui_imports: HashSet::new(),
            source_map,
            cwd,
        }
    }

    fn jsx_element_name_str(name: &JSXElementName) -> Option<String> {
        match name {
            JSXElementName::Ident(i) => Some(i.sym.to_string()),
            JSXElementName::JSXMemberExpr(m) => {
                // e.g. React.Button -> "Button" (prop is IdentName)
                Some(m.prop.sym.to_string())
            }
            JSXElementName::JSXNamespacedName(n) => Some(n.name.sym.to_string()),
            #[cfg(swc_ast_unknown)]
            _ => panic!("unknown JSXElementName"),
        }
    }

    fn has_data_source(attrs: &[JSXAttrOrSpread]) -> bool {
        attrs.iter().any(|a| {
            if let JSXAttrOrSpread::JSXAttr(attr) = a {
                if let JSXAttrName::Ident(i) = &attr.name {
                    return i.sym == "data-source";
                }
            }
            false
        })
    }

    fn make_data_source_attr(&self, span: swc_core::common::Span) -> Option<JSXAttrOrSpread> {
        if span.is_dummy() {
            return None;
        }
        let loc = self.source_map.lookup_char_pos(span.lo);
        let line = loc.line;
        let filename = loc.file.name.to_string().replace('\\', "/");
        let relative = self
            .cwd
            .as_ref()
            .map(|cwd| relative_path(cwd, &filename))
            .unwrap_or(filename);
        let source_value = format!("{relative}:{line}");
        let value = Str {
            span: DUMMY_SP,
            value: source_value.into(),
            raw: None,
        };
        let attr = JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(
                Ident::new("data-source".into(), DUMMY_SP, SyntaxContext::empty()).into(),
            ),
            value: Some(JSXAttrValue::Lit(Lit::Str(value))),
        };
        Some(JSXAttrOrSpread::JSXAttr(attr))
    }
}

impl VisitMut for ReactSourceStringVisitor {
    fn visit_mut_import_decl(&mut self, decl: &mut ImportDecl) {
        let source = decl.src.value.to_string();
        let in_libs = self.libraries.contains(&source)
            || self
                .libraries
                .contains(source.split('/').next().unwrap_or(""));
        if in_libs {
            for spec in &decl.specifiers {
                let local = match spec {
                    ImportSpecifier::Named(s) => &s.local,
                    ImportSpecifier::Default(s) => &s.local,
                    ImportSpecifier::Namespace(s) => &s.local,
                };
                let name = local.sym.to_string();
                if !self.excluded.contains(&name.to_lowercase()) {
                    self.ui_imports.insert(name);
                }
            }
        }
        decl.visit_mut_children_with(self);
    }

    fn visit_mut_jsx_opening_element(&mut self, el: &mut JSXOpeningElement) {
        el.visit_mut_children_with(self);

        let element_name = match Self::jsx_element_name_str(&el.name) {
            Some(n) => n,
            None => return,
        };
        let name_lower = element_name.to_lowercase();
        if self.excluded.contains(&name_lower) {
            return;
        }
        let is_lowercase = element_name == name_lower;
        let is_ui_import = self.ui_imports.contains(&element_name);
        if !is_lowercase && !is_ui_import {
            return;
        }

        if Self::has_data_source(&el.attrs) {
            return;
        }

        if let Some(attr) = self.make_data_source_attr(el.span) {
            el.attrs.push(attr);
        }
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let (libraries, excluded) = parse_config(&metadata);
    let source_map = metadata.source_map.clone();
    let mut visitor = ReactSourceStringVisitor::new(libraries, excluded, source_map, &metadata);
    let mut program = program;
    program.visit_mut_with(&mut visitor);
    program
}
