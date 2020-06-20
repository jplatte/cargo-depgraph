use std::{
    path::{Path, PathBuf},
    process::exit,
};

use cargo_lock::Lockfile;
use cargo_metadata::MetadataCommand;
use petgraph::dot::{Config, Dot};

fn main() -> anyhow::Result<()> {
    let lockfile = load_lockfile(&None);
    let tree = lockfile.dependency_tree()?;

    let metadata = MetadataCommand::new().no_deps().exec()?;
    let root_pkgs: Vec<_> =
        metadata.workspace_members.iter().map(|pkg_id| &metadata[pkg_id]).collect();

    println!(
        "{:?}",
        Dot::with_attr_getters(
            &tree.graph(),
            &[Config::EdgeNoLabel],
            &|_, _edge| { format!("") },
            &|_, node| {
                let pkg = node.1;
                let mut attrs = vec![format!("label = \"{}\"", pkg.name)];

                let is_root_pkg = root_pkgs.iter().any(|root_pkg| {
                    pkg.name.as_str() == root_pkg.name
                        && pkg.version == root_pkg.version
                        && pkg.source.as_ref().map(|s| s.to_string()).as_ref()
                            == root_pkg.source.as_ref().map(|s| &s.repr)
                });
                if is_root_pkg {
                    attrs.push("shape = box".to_owned());
                }

                attrs.join(",")
            }
        )
    );

    Ok(())
}

/// Load a lockfile from the given path (or `Cargo.toml`)
fn load_lockfile(path: &Option<PathBuf>) -> Lockfile {
    let path = path.as_ref().map(AsRef::as_ref).unwrap_or_else(|| Path::new("Cargo.lock"));

    Lockfile::load(path).unwrap_or_else(|e| {
        eprintln!("*** error: {}", e);
        exit(1);
    })
}
