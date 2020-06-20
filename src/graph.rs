use std::collections::{hash_map::Entry as HashMapEntry, HashMap, VecDeque};

use anyhow::Context;
use cargo_metadata::{Metadata, NodeDep, Package as MetaPackage, PackageId, Resolve};
use petgraph::graph::{DiGraph, NodeIndex};

use crate::package::Package;

pub type DepGraph = DiGraph<Package, NodeDep, u16>;

pub fn get_dep_graph(metadata: Metadata) -> anyhow::Result<DepGraph> {
    let mut builder = DepGraphBuilder::new(metadata)?;
    builder.add_workspace_members()?;
    builder.add_dependencies()?;

    Ok(builder.graph)
}

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
    packages: Vec<Option<MetaPackage>>,
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
            packages: metadata.packages.into_iter().map(Some).collect(),
            resolve,
        })
    }

    fn add_workspace_members(&mut self) -> anyhow::Result<()> {
        for pkg_id in &self.workspace_members {
            let pkg =
                pop_package(&mut self.packages, pkg_id).context("package not found in packages")?;
            let node_idx = self.graph.add_node(Package::new(pkg, true));
            self.deps_add_queue.push_back(pkg_id.clone());
            let old_val = self.node_indices.insert(pkg_id.clone(), node_idx);
            assert!(old_val.is_none());
        }

        Ok(())
    }

    fn add_dependencies(&mut self) -> anyhow::Result<()> {
        while let Some(pkg_id) = self.deps_add_queue.pop_front() {
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
                let mut packages = &mut self.packages;
                let child_idx = match self.node_indices.entry(dep.pkg.clone()) {
                    HashMapEntry::Occupied(o) => *o.get(),
                    HashMapEntry::Vacant(v) => {
                        let pkg = pop_package(&mut packages, &dep.pkg).unwrap();
                        let idx = self.graph.add_node(Package::new(pkg, false));
                        self.deps_add_queue.push_back(dep.pkg.clone());
                        v.insert(idx);
                        idx
                    }
                };

                self.graph.add_edge(parent_idx, child_idx, dep.clone());
            }
        }

        Ok(())
    }
}

fn pop_package(packages: &mut [Option<MetaPackage>], pkg_id: &PackageId) -> Option<MetaPackage> {
    packages.iter_mut().find_map(|op| match op {
        Some(p) if p.id == *pkg_id => op.take(),
        _ => None,
    })
}
