use std::collections::{hash_map::Entry as HashMapEntry, HashMap, VecDeque};

use anyhow::Context;
use cargo_metadata::{
    DependencyKind as MetaDepKind, Metadata, Package as MetaPackage, PackageId, Resolve,
};
use petgraph::stable_graph::NodeIndex;

use super::DepGraph;
use crate::{
    cli::Config,
    dep_info::{DepInfo, DepKind},
    package::Package,
};

pub struct DepGraphBuilder {
    /// The dependency graph being built.
    pub graph: DepGraph,
    /// Map from PackageId to graph node index.
    node_indices: HashMap<PackageId, NodeIndex<u16>>,
    /// Queue of packages whose dependencies still need to be added to the graph.
    deps_add_queue: VecDeque<PackageId>,

    /// Workspace members, obtained from cargo_metadata.
    workspace_members: Vec<PackageId>,
    /// Package info obtained from cargo_metadata. To be transformed into graph nodes.
    packages: Vec<MetaPackage>,
    /// The dependency graph obtained from cargo_metadata. To be transformed into graph edges.
    resolve: Resolve,
}

impl DepGraphBuilder {
    pub fn new(metadata: Metadata) -> anyhow::Result<Self> {
        let resolve = metadata
            .resolve
            .context("Couldn't obtain dependency graph. Your cargo version may be too old.")?;

        Ok(Self {
            graph: DepGraph::with_capacity(
                resolve.nodes.len(),
                resolve.nodes.iter().map(|n| n.deps.len()).sum(),
            ),
            node_indices: HashMap::new(),
            deps_add_queue: VecDeque::new(),

            workspace_members: metadata.workspace_members,
            packages: metadata.packages,
            resolve,
        })
    }

    pub fn add_workspace_members(&mut self) -> anyhow::Result<()> {
        for pkg_id in &self.workspace_members {
            let pkg = get_package(&self.packages, pkg_id);
            let node_idx = self.graph.add_node(Package::new(pkg, true));
            self.deps_add_queue.push_back(pkg_id.clone());
            let old_val = self.node_indices.insert(pkg_id.clone(), node_idx);
            assert!(old_val.is_none());
        }

        Ok(())
    }

    pub fn add_dependencies(&mut self, config: &Config) -> anyhow::Result<()> {
        while let Some(pkg_id) = self.deps_add_queue.pop_front() {
            let parent_idx = *self
                .node_indices
                .get(&pkg_id)
                .context("trying to add deps of package that's not in the graph")?;

            let extra_dep_info = get_package(&self.packages, &pkg_id).dependencies.clone();

            let resolve_node = self
                .resolve
                .nodes
                .iter()
                .find(|n| n.id == pkg_id)
                .context("package not found in resolve")?;

            for dep in &resolve_node.deps {
                if dep.dep_kinds.iter().all(|i| skip_dep(config, i)) {
                    continue;
                }

                let pkg = get_package(&self.packages, &dep.pkg);
                let is_proc_macro = is_proc_macro(pkg);

                if is_proc_macro && !config.build_deps {
                    continue;
                }

                let child_idx = match self.node_indices.entry(dep.pkg.clone()) {
                    HashMapEntry::Occupied(o) => *o.get(),
                    HashMapEntry::Vacant(v) => {
                        let idx = self.graph.add_node(Package::new(pkg, false));
                        self.deps_add_queue.push_back(dep.pkg.clone());
                        v.insert(idx);
                        idx
                    }
                };

                for info in &dep.dep_kinds {
                    let extra = extra_dep_info.iter().find(|d| {
                        // `dep.name` is not the source crate name but the one used for that
                        // dependency in the parent, so if the dependency is renamed, we need to use
                        // the alternative name.
                        let name = d.rename.as_ref().unwrap_or(&d.name);

                        *name == dep.name
                            && d.kind == info.kind
                            && d.target.as_ref().map(|t| t.to_string())
                                == info.target.as_ref().map(|t| t.to_string())
                    });
                    let is_optional = extra.map(|dep| dep.optional).unwrap_or(false);

                    // We checked whether to skip this dependency fully above, but if there's
                    // multiple dependencies from A to B (e.g. normal dependency with no features,
                    // dev-dependency with some features activated), we might have to skip adding
                    // some of the edges.
                    if !skip_dep(config, info) {
                        self.graph.add_edge(
                            parent_idx,
                            child_idx,
                            DepInfo {
                                kind: if is_proc_macro { DepKind::BUILD } else { info.kind.into() },
                                is_target_dep: info.target.is_some(),
                                is_optional,
                                is_optional_direct: is_optional,
                                visited: false,
                            },
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

fn get_package<'a>(packages: &'a [MetaPackage], pkg_id: &PackageId) -> &'a MetaPackage {
    packages.iter().find(|pkg| pkg.id == *pkg_id).unwrap()
}

fn is_proc_macro(pkg: &MetaPackage) -> bool {
    let res = pkg.targets.iter().any(|t| t.kind.iter().any(|k| k == "proc-macro"));
    if res && pkg.targets.iter().any(|t| t.kind.iter().any(|k| k == "lib")) {
        eprintln!("enountered a crate that is both a regular library and a proc-macro");
    }

    res
}

pub fn skip_dep(config: &Config, info: &cargo_metadata::DepKindInfo) -> bool {
    (!config.build_deps && info.kind == MetaDepKind::Build)
        || (!config.dev_deps && info.kind == MetaDepKind::Development)
        || (!config.target_deps && info.target.is_some())
}
