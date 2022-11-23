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

pub fn get_dep_graph(metadata: Metadata, config: &Config) -> anyhow::Result<DepGraph> {
    let mut builder = DepGraphBuilder::new(metadata)?;
    builder.add_workspace_members(config)?;
    builder.add_dependencies(config)?;

    Ok(builder.graph)
}

struct DepGraphBuilder {
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

    pub fn add_workspace_members(&mut self, config: &Config) -> anyhow::Result<()> {
        for pkg_id in &self.workspace_members {
            let pkg = get_package(&self.packages, pkg_id);
            if config.exclude.contains(&pkg.name) {
                continue;
            }

            let node_idx = self.graph.add_node(Package::new(pkg, true));
            self.deps_add_queue.push_back(pkg_id.clone());
            let old_val = self.node_indices.insert(pkg_id.clone(), node_idx);
            assert!(old_val.is_none());
        }

        Ok(())
    }

    pub fn add_dependencies(&mut self, config: &Config) -> anyhow::Result<()> {
        while let Some(pkg_id) = self.deps_add_queue.pop_front() {
            let pkg = get_package(&self.packages, &pkg_id);

            let parent_idx = *self
                .node_indices
                .get(&pkg_id)
                .context("trying to add deps of package that's not in the graph")?;

            let resolve_node = self
                .resolve
                .nodes
                .iter()
                .find(|n| n.id == pkg_id)
                .context("package not found in resolve")?;

            for dep in &resolve_node.deps {
                // Same as dep.name in most cases, but not if it got renamed in parent's Cargo.toml
                let dep_crate_name = &get_package(&self.packages, &dep.pkg).name;

                if config.exclude.contains(dep_crate_name)
                    || dep.dep_kinds.iter().all(|i| skip_dep(config, i))
                {
                    continue;
                }

                let child_idx = match self.node_indices.entry(dep.pkg.clone()) {
                    HashMapEntry::Occupied(o) => *o.get(),
                    HashMapEntry::Vacant(v) => {
                        if config.workspace_only {
                            // For workspace-only mode, all the packages we care to render are
                            // already added to node_indices by add_workspace_members.
                            continue;
                        }

                        let idx = self
                            .graph
                            .add_node(Package::new(get_package(&self.packages, &dep.pkg), false));
                        self.deps_add_queue.push_back(dep.pkg.clone());
                        v.insert(idx);
                        idx
                    }
                };

                let child_is_proc_macro = self.graph[child_idx].dep_info.kind == DepKind::BUILD;

                for info in &dep.dep_kinds {
                    let extra = pkg.dependencies.iter().find(|d| {
                        d.name == *dep_crate_name
                            && d.kind == info.kind
                            && d.target.as_ref().map(|t| t.to_string())
                                == info.target.as_ref().map(|t| t.to_string())
                    });
                    let is_optional = match extra {
                        Some(dep) => dep.optional,
                        None => {
                            eprintln!(
                                "dependency {} of {} not found in packages \
                                 => dependencies, this should never happen!",
                                dep_crate_name, pkg.name,
                            );
                            false
                        }
                    };

                    // We checked whether to skip this dependency fully above, but if there's
                    // multiple dependencies from A to B (e.g. normal dependency with no features,
                    // dev-dependency with some features activated), we might have to skip adding
                    // some of the edges.
                    if !skip_dep(config, info) {
                        self.graph.add_edge(
                            parent_idx,
                            child_idx,
                            DepInfo {
                                kind: DepKind::new(info.kind, child_is_proc_macro),
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

pub fn skip_dep(config: &Config, info: &cargo_metadata::DepKindInfo) -> bool {
    (!config.build_deps && info.kind == MetaDepKind::Build)
        || (!config.dev_deps && info.kind == MetaDepKind::Development)
        || (!config.target_deps && info.target.is_some())
}
