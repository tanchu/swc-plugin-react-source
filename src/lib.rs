//! SWC plugin: adds `data-source="path:line"` to JSX elements (HTML elements and components).
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
    libraries: Option<Vec<String>>,
    prefer_call_site: Option<bool>,
}

#[derive(Default)]
struct ParsedConfig {
    excluded: HashSet<String>,
    root: Option<String>,
    libraries: Vec<String>,
    prefer_call_site: bool,
}

fn parse_config(metadata: &TransformPluginProgramMetadata) -> ParsedConfig {
    let config_str = match metadata.get_transform_plugin_config() {
        Some(s) => s,
        None => return ParsedConfig::default(),
    };
    let config: PluginConfig = match serde_json::from_str(&config_str) {
        Ok(c) => c,
        Err(_) => return ParsedConfig::default(),
    };
    ParsedConfig {
        excluded: config
            .excluded
            .map(|v| v.into_iter().map(|s| s.to_lowercase()).collect())
            .unwrap_or_default(),
        root: config.root,
        libraries: config.libraries.unwrap_or_default(),
        prefer_call_site: config.prefer_call_site.unwrap_or(false),
    }
}

/// Same rule as the Babel and Vite plugins: the package itself or any of its subpaths.
fn from_library(source: &str, libraries: &[String]) -> bool {
    libraries
        .iter()
        .any(|lib| source == lib || source.starts_with(&format!("{lib}/")))
}

/// Lowercase tags (`<div>`, `<svg:rect>`) are HTML/SVG elements; everything else is a component.
fn is_html_element(name: &JSXElementName) -> bool {
    match name {
        JSXElementName::Ident(i) => i.sym.chars().next().is_some_and(|c| c.is_lowercase()),
        JSXElementName::JSXNamespacedName(_) => true,
        _ => false,
    }
}

/// The identifier that decides whether the element is a component: `UI` for `<UI.Button>`.
fn root_name(name: &JSXElementName) -> Option<String> {
    match name {
        JSXElementName::Ident(i) => Some(i.sym.to_string()),
        JSXElementName::JSXNamespacedName(n) => Some(n.ns.sym.to_string()),
        JSXElementName::JSXMemberExpr(m) => {
            let mut object = &m.obj;
            loop {
                match object {
                    JSXObject::JSXMemberExpr(inner) => object = &inner.obj,
                    JSXObject::Ident(i) => return Some(i.sym.to_string()),
                    #[cfg(swc_ast_unknown)]
                    _ => return None,
                }
            }
        }
        #[cfg(swc_ast_unknown)]
        _ => None,
    }
}

fn library_imports(module: &Module, libraries: &[String]) -> HashSet<String> {
    let mut imports = HashSet::new();
    for item in &module.body {
        if let ModuleItem::ModuleDecl(ModuleDecl::Import(decl)) = item {
            if !from_library(&decl.src.value.to_string_lossy(), libraries) {
                continue;
            }
            for specifier in &decl.specifiers {
                let local = match specifier {
                    ImportSpecifier::Named(s) => &s.local,
                    ImportSpecifier::Default(s) => &s.local,
                    ImportSpecifier::Namespace(s) => &s.local,
                    #[cfg(swc_ast_unknown)]
                    _ => continue,
                };
                imports.insert(local.sym.to_string());
            }
        }
    }
    imports
}

fn strip_virtual_prefix(filename: &str) -> &str {
    if filename.starts_with('[') {
        if let Some(pos) = filename.find("]/") {
            return &filename[pos + 2..];
        }
    }
    filename
}

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
    libraries: Vec<String>,
    library_imports: Option<HashSet<String>>,
    /// Insert the attribute before spread props, so a `data-source` passed in from the parent wins.
    prefer_call_site: bool,
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
            libraries: config.libraries,
            library_imports: None,
            prefer_call_site: config.prefer_call_site,
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
        let raw_filename = loc.file.name.to_string().replace('\\', "/");
        let filename = strip_virtual_prefix(&raw_filename);
        let relative = self
            .cwd
            .as_ref()
            .map(|cwd| relative_path(cwd, filename))
            .unwrap_or_else(|| filename.to_string());
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
            value: Some(JSXAttrValue::Str(value)),
        };
        Some(JSXAttrOrSpread::JSXAttr(attr))
    }
}

impl VisitMut for ReactSourceStringVisitor {
    fn visit_mut_module(&mut self, module: &mut Module) {
        if !self.libraries.is_empty() {
            self.library_imports = Some(library_imports(module, &self.libraries));
        }
        module.visit_mut_children_with(self);
    }

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

        if let Some(imports) = &self.library_imports {
            let from_imports = root_name(&el.name).is_some_and(|n| imports.contains(&n));
            if !is_html_element(&el.name) && !from_imports {
                return;
            }
        }

        if let Some(attr) = self.make_data_source_attr(el.span) {
            if self.prefer_call_site {
                el.attrs.insert(0, attr);
            } else {
                el.attrs.push(attr);
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_library_matches_package_and_subpaths() {
        let libs = vec!["@mui/material".to_string()];
        assert!(from_library("@mui/material", &libs));
        assert!(from_library("@mui/material/Button", &libs));
        assert!(!from_library("@mui/material-next", &libs));
        assert!(from_library("@mui/icons-material", &["@mui".to_string()]));
    }

    #[test]
    fn relative_path_parity() {
        assert_eq!(relative_path("/proj/", "/proj/src/a.tsx"), "src/a.tsx");
        assert_eq!(relative_path("/other", "/proj/a.tsx"), "/proj/a.tsx");
    }
}
