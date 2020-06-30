use std::collections::{hash_map::Entry as HashMapEntry, HashMap, VecDeque};

use anyhow::Context;
use cargo_metadata::{
    DependencyKind as MetaDepKind, Metadata, Package as MetaPackage, PackageId, Resolve,
};
use petgraph::{
    algo::all_simple_paths,
    stable_graph::{NodeIndex, StableDiGraph},
    Direction,
};

use crate::{cli::Config, dep_info::DepInfo, package::Package};

pub type DepGraph = StableDiGraph<Package, DepInfo, u16>;

pub fn get_dep_graph(metadata: Metadata, config: &Config) -> anyhow::Result<DepGraph> {
    let mut builder = DepGraphBuilder::new(metadata)?;
    builder.add_workspace_members()?;
    builder.add_dependencies(config)?;

    Ok(builder.graph)
}

pub fn update_dep_info(graph: &mut DepGraph) {
    for idx in graph.node_indices().collect::<Vec<_>>() {
        // We're only mutating nodes, not adding or deleting them, so we can safely use the indices
        // that were collected at the start thoughout to visit each node once (or more than once,
        // in case we recurse inside update_node).
        update_node(graph, idx);
    }
}

fn update_node(graph: &mut DepGraph, idx: NodeIndex<u16>) {
    // Special case for workspace members
    if graph[idx].dep_info.is_none() {
        let mut outgoing = graph.neighbors_directed(idx, Direction::Outgoing).detach();
        while let Some(edge_idx) = outgoing.next_edge(graph) {
            graph[edge_idx].visited = true;
        }

        return;
    }

    let mut incoming = graph.neighbors_directed(idx, Direction::Incoming).detach();
    let mut node_info: Option<DepInfo> = None;
    while let Some((edge_idx, node_idx)) = incoming.next(graph) {
        if !graph[edge_idx].visited {
            update_node(graph, node_idx);
        }

        let edge_info = graph[edge_idx];
        assert!(edge_info.visited);

        if let Some(i) = &mut node_info {
            i.is_target_dep &= edge_info.is_target_dep;
            i.is_optional &= edge_info.is_optional;
            i.kind.combine_incoming(edge_info.kind);
        } else {
            node_info = Some(edge_info);
        }
    }

    let node_info = node_info.expect("non-workspace members to have at least one incoming edge");
    graph[idx].dep_info = Some(node_info);

    let mut outgoing = graph.neighbors_directed(idx, Direction::Outgoing).detach();
    while let Some(edge_idx) = outgoing.next_edge(graph) {
        let edge_info = &mut graph[edge_idx];

        // it's unclear to me why this happens... maybe a bug in petgraph?
        if edge_info.visited {
            continue;
        }

        edge_info.visited = true;
        edge_info.is_target_dep |= node_info.is_target_dep;
        edge_info.is_optional |= node_info.is_optional;
        edge_info.kind.update_outgoing(node_info.kind);
    }
}

pub fn dedup_transitive_deps(graph: &mut DepGraph) {
    // this can probably be optimized.
    // maybe it would make sense to make this less conservative about what to remove.

    for idx in graph.node_indices().collect::<Vec<_>>() {
        // We're only removing nodes, not adding new ones, so we can use the node indices collected
        // at the start as long as we check that they're still valid within the current graph.
        if !graph.contains_node(idx) {
            continue;
        }

        let mut outgoing = graph.neighbors_directed(idx, Direction::Outgoing).detach();
        while let Some((edge_idx, node_idx)) = outgoing.next(graph) {
            let any_paths =
                all_simple_paths::<Vec<_>, _>(&*graph, idx, node_idx, 1, None).next().is_some();

            if any_paths {
                graph.remove_edge(edge_idx);
            }

            // Previous more conversative and also buggy version
            /*if graph.neighbors_directed(node_idx, Direction::Incoming).count() < 2 {
                // graph[idx] is the only node that depends on graph[node_idx], do nothing
                break;
            }

            let node_kind = graph[idx].dep_kind();
            let paths: Vec<_> =
                all_simple_paths::<Vec<_>, _>(&*graph, idx, node_idx, 1, None).collect();
            if paths.iter().any(|path| path.iter().all(|&i| graph[i].dep_kind() == node_kind)) {
                graph.remove_edge(edge_idx);
            }*/
        }
    }
}

// TODO: Clone DepKindInfo to be able to distinguish build-dep of test-dep from just test-dep

struct DepGraphBuilder {
    /// The dependency graph being built.
    graph: DepGraph,
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
    fn new(metadata: Metadata) -> anyhow::Result<Self> {
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

    fn add_workspace_members(&mut self) -> anyhow::Result<()> {
        for pkg_id in &self.workspace_members {
            let pkg = get_package(&self.packages, pkg_id);
            let node_idx = self.graph.add_node(Package::new(pkg, true));
            self.deps_add_queue.push_back(pkg_id.clone());
            let old_val = self.node_indices.insert(pkg_id.clone(), node_idx);
            assert!(old_val.is_none());
        }

        Ok(())
    }

    fn add_dependencies(&mut self, config: &Config) -> anyhow::Result<()> {
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

                let child_idx = match self.node_indices.entry(dep.pkg.clone()) {
                    HashMapEntry::Occupied(o) => *o.get(),
                    HashMapEntry::Vacant(v) => {
                        let pkg = get_package(&self.packages, &dep.pkg);
                        let idx = self.graph.add_node(Package::new(&pkg, false));
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
                                kind: info.kind.into(),
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
    (!config.normal_deps && info.kind == MetaDepKind::Normal)
        || (!config.build_deps && info.kind == MetaDepKind::Build)
        || (!config.dev_deps && info.kind == MetaDepKind::Development)
        || (!config.target_deps && info.target.is_some())
}
