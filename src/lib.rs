use std::sync::OnceLock;

use regex::Regex;
use serde::Deserialize;
use swc_core::ecma::{
    ast::{Program, Str},
    visit::{as_folder, FoldWith, VisitMut},
};
use swc_core::plugin::{
    metadata::TransformPluginMetadataContextKind, plugin_transform,
    proxies::TransformPluginProgramMetadata,
};

pub struct TransformVisitor {
    config: Config,
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

impl VisitMut for TransformVisitor {
    fn visit_mut_import_decl(&mut self, n: &mut swc_core::ecma::ast::ImportDecl) {
        let import_source = &n.src.as_ref().value;

        static REGEX_CELL: OnceLock<Regex> = OnceLock::new();
        let rootpath_regex = REGEX_CELL.get_or_init(|| {
            let allowed_pathnames = self
                .config
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
                self.config
                    .prefix
                    .as_ref()
                    .map(|prefix| prefix.as_str())
                    .unwrap_or("@"),
            );
            Regex::new(&format!(r"{}({})?/", prefix, allowed_pathnames)).unwrap()
        });

        if let Some(result) = rootpath_regex.captures(&import_source) {
            let replace_target = result.get(0).unwrap();
            let alias_subpath = result.get(1);

            let relative_path = self.filename.replace(&self.cwd, "");
            let last_src_pos = relative_path.rfind("src/");
            if last_src_pos.is_none() {
                return;
            }

            let closest_src_path = &relative_path[0..last_src_pos.unwrap() + 4];
            let import_payload = import_source.replace(replace_target.as_str(), "");

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

            let src_mut = n.src.as_mut();
            src_mut.value = fixed_path_atom.clone();
            src_mut.raw = Some(fixed_path_atom);
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

    program.fold_with(&mut as_folder(TransformVisitor {
        config,
        filename,
        cwd,
    }))
}
