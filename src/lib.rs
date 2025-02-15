use std::sync::OnceLock;

use regex::Regex;
use serde::Deserialize;
use swc_core::ecma::{
    ast::{Callee, ImportPhase, Program, Str},
    visit::{visit_mut_pass, VisitMut},
};
use swc_core::plugin::{
    metadata::TransformPluginMetadataContextKind, plugin_transform,
    proxies::TransformPluginProgramMetadata,
};

pub struct TransformVisitor {
    filename: String,
    cwd: String,
}

#[derive(Deserialize, Default)]
pub struct Config {
    /// Prefix of import. Default is `@`.
    pub prefix: Option<String>,

    /// Allowed pathnames after prefix, for example `common` can resolve `@common/`
    #[serde(rename = "allowedPathnames")]
    pub allowed_pathnames: Option<Vec<String>>,
}

static REGEX_CELL: OnceLock<Regex> = OnceLock::new();

impl TransformVisitor {
    fn fix_path(&self, target: &mut Str) {
        let source = target.value.clone();
        let rootpath_regex = REGEX_CELL.get().unwrap();

        if let Some(result) = rootpath_regex.captures(&source) {
            let replace_target = result.get(0).unwrap();
            let alias_subpath = result.get(1);

            let relative_path = self.filename.replace(&self.cwd, "");
            let last_src_pos = relative_path.rfind("src/");
            if last_src_pos.is_none() {
                return;
            }

            let closest_src_path = &relative_path[0..last_src_pos.unwrap() + 4];
            let import_payload = source.replace(replace_target.as_str(), "");

            let fixed_path = match alias_subpath {
                Some(path) => format!(
                    "\"{}{}/{}\"",
                    closest_src_path,
                    path.as_str(),
                    import_payload
                ),
                _ => format!("\"{}{}\"", closest_src_path, import_payload),
            };

            let fixed_path_atom = Str::from_tpl_raw(&fixed_path);

            target.value = fixed_path_atom.clone();
            target.raw = Some(fixed_path_atom);
        }
    }
}

impl VisitMut for TransformVisitor {
    /// export { foo } from './foo';
    fn visit_mut_named_export(&mut self, n: &mut swc_core::ecma::ast::NamedExport) {
        match &mut n.src {
            Some(target) => self.fix_path(target.as_mut()),
            None => {}
        };
    }

    /// export * from './foo';
    fn visit_mut_export_all(&mut self, n: &mut swc_core::ecma::ast::ExportAll) {
        self.fix_path(n.src.as_mut());
    }

    /// import { foo } from './foo';
    fn visit_mut_import_decl(&mut self, n: &mut swc_core::ecma::ast::ImportDecl) {
        self.fix_path(n.src.as_mut());
    }

    /// () => import('./foo'),
    fn visit_mut_call_expr(&mut self, n: &mut swc_core::ecma::ast::CallExpr) {
        match n.callee {
            Callee::Import(callee) => {
                // ignore non import() function call and empty arguments
                if callee.phase != ImportPhase::Evaluation || n.args.len() <= 0 {
                    return;
                }

                // extract path literal ('./foo')
                let import_source = &mut n.args[0];
                let literal = import_source.expr.as_mut().as_mut_lit();
                if literal.is_none() {
                    return;
                }
                let target = literal.unwrap().as_mut_str();
                if target.is_none() {
                    return;
                }
                // do fix path logics
                self.fix_path(target.unwrap());
            }
            _ => {}
        }
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let filename = metadata
        .get_context(&TransformPluginMetadataContextKind::Filename)
        .unwrap();
    let cwd = metadata
        .get_context(&TransformPluginMetadataContextKind::Cwd)
        .unwrap();
    let config: Config = serde_json::from_str(
        &metadata
            .get_transform_plugin_config()
            .expect("failed to get plugin config"),
    )
    .unwrap_or_default();

    REGEX_CELL.get_or_init(|| {
        let allowed_pathnames = config
            .allowed_pathnames
            .as_ref()
            .map(|pathnames| {
                pathnames
                    .into_iter()
                    .map(|pathname| regex::escape(pathname))
                    .collect::<Vec<String>>()
                    .join("|")
            })
            .unwrap_or(String::from(""));

        let prefix = regex::escape(
            config
                .prefix
                .as_ref()
                .map(|prefix| prefix.as_str())
                .unwrap_or("@"),
        );
        Regex::new(&format!(r"{}({})?/", prefix, allowed_pathnames)).unwrap()
    });

    program.apply(visit_mut_pass(TransformVisitor { filename, cwd }))
}
