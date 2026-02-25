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
    excluded: Option<Vec<String>>,
    root: Option<String>,
}

struct ParsedConfig {
    excluded: HashSet<String>,
    root: Option<String>,
}

fn parse_config(metadata: &TransformPluginProgramMetadata) -> ParsedConfig {
    let config_str = match metadata.get_transform_plugin_config() {
        Some(s) => s,
        None => {
            return ParsedConfig {
                excluded: HashSet::new(),
                root: None,
            }
        }
    };
    let config: PluginConfig = match serde_json::from_str(&config_str) {
        Ok(c) => c,
        Err(_) => {
            return ParsedConfig {
                excluded: HashSet::new(),
                root: None,
            }
        }
    };
    ParsedConfig {
        excluded: config
            .excluded
            .map(|v| v.into_iter().map(|s| s.to_lowercase()).collect())
            .unwrap_or_default(),
        root: config.root,
    }
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
    excluded: HashSet<String>,
    source_map: swc_core::plugin::proxies::PluginSourceMapProxy,
    cwd: Option<String>,
}

impl ReactSourceStringVisitor {
    fn new(
        config: ParsedConfig,
        source_map: swc_core::plugin::proxies::PluginSourceMapProxy,
        metadata: &TransformPluginProgramMetadata,
    ) -> Self {
        let cwd = config.root.filter(|s| !s.is_empty()).or_else(|| {
            metadata
                .get_experimental_context("cwd")
                .filter(|s| !s.is_empty())
        });
        Self {
            excluded: config.excluded,
            source_map,
            cwd,
        }
    }

    fn jsx_element_name_str(name: &JSXElementName) -> Option<String> {
        match name {
            JSXElementName::Ident(i) => Some(i.sym.to_string()),
            JSXElementName::JSXMemberExpr(m) => Some(m.prop.sym.to_string()),
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
    fn visit_mut_jsx_opening_element(&mut self, el: &mut JSXOpeningElement) {
        el.visit_mut_children_with(self);

        let element_name = match Self::jsx_element_name_str(&el.name) {
            Some(n) => n,
            None => return,
        };

        if self.excluded.contains(&element_name.to_lowercase()) {
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
    let config = parse_config(&metadata);
    let source_map = metadata.source_map.clone();
    let mut visitor = ReactSourceStringVisitor::new(config, source_map, &metadata);
    let mut program = program;
    program.visit_mut_with(&mut visitor);
    program
}
