use std::collections::{hash_map::Entry as HashMapEntry, HashMap, VecDeque};

use anyhow::Context;
use cargo_metadata::{DependencyKind as MetaDepKind, Metadata, Package as MetaPackage, PackageId};

use super::DepGraph;
use crate::{
    cli::Config,
    dep_info::{DepInfo, DepKind},
    package::Package,
    util::is_proc_macro,
};

pub(crate) fn get_dep_graph(metadata: Metadata, config: &Config) -> anyhow::Result<DepGraph> {
    let resolve = metadata
        .resolve
        .context("Couldn't obtain dependency graph. Your cargo version may be too old.")?;

    let mut graph = DepGraph::with_capacity(
        resolve.nodes.len(),
        resolve.nodes.iter().map(|n| n.deps.len()).sum(),
    );

    // Map from PackageId to graph node index.
    let mut node_indices = HashMap::new();

    // Queue of packages whose dependencies still need to be added to the graph.
    let mut deps_add_queue = VecDeque::new();

    // Add roots
    for pkg_id in &metadata.workspace_members {
        let pkg = get_package(&metadata.packages, pkg_id);

        // Roots are specified explicitly and don't contain this package
        if (!config.root.is_empty() && !config.root.contains(&pkg.name))
            // Excludes are specified and include this package
            || config.exclude.contains(&pkg.name)
            // Includes are specified and do not include this package
            || (!config.include.is_empty() && !config.include.contains(&pkg.name))
            // Build dependencies are disabled and this package is a proc-macro
            || !config.build_deps && is_proc_macro(pkg)
        {
            continue;
        }

        let node_idx = graph.add_node(Package::new(pkg, true));
        deps_add_queue.push_back((pkg_id.clone(), 0u32));
        let old_val = node_indices.insert(pkg_id.clone(), node_idx);
        assert!(old_val.is_none());
    }

    // Add dependencies of the roots
    while let Some((pkg_id, depth)) = deps_add_queue.pop_front() {
        let pkg = get_package(&metadata.packages, &pkg_id);

        let parent_idx = *node_indices
            .get(&pkg_id)
            .context("trying to add deps of package that's not in the graph")?;

        let resolve_node = resolve
            .nodes
            .iter()
            .find(|n| n.id == pkg_id)
            .context("package not found in resolve")?;

        for dep in &resolve_node.deps {
            // Same as dep.name in most cases, but not if it got renamed in parent's Cargo.toml
            let dep_crate_name = &get_package(&metadata.packages, &dep.pkg).name;

            // Excludes are specified and include this package
            if config.exclude.contains(dep_crate_name)
                // Includes are specified and do not include this package
                || (!config.include.is_empty() && !config.include.contains(dep_crate_name))
                // This dependency should be skipped because of its dep_kinds
                || dep.dep_kinds.iter().all(|i| skip_dep(config, i))
            {
                continue;
            }

            let child_idx = match node_indices.entry(dep.pkg.clone()) {
                HashMapEntry::Occupied(o) => *o.get(),
                HashMapEntry::Vacant(v) => {
                    let is_workspace_member = metadata.workspace_members.contains(&dep.pkg);

                    // For workspace-only mode, don't add non-workspace
                    // dependencies to deps_add_queue or node_indices.
                    if config.workspace_only && !is_workspace_member {
                        continue;
                    }

                    // Don't add dependencies of dependencies if we're at the depth limit
                    if depth + 1 > config.depth.unwrap_or(u32::MAX) {
                        continue;
                    }

                    let dep_pkg = &get_package(&metadata.packages, &dep.pkg);
                    let dep_pkg = Package::new(dep_pkg, is_workspace_member);

                    // proc-macros are a bit weird because Cargo doesn't report
                    // them as build dependencies when really they are.
                    if !config.build_deps && dep_pkg.is_proc_macro {
                        continue;
                    }

                    let idx = graph.add_node(dep_pkg);

                    deps_add_queue.push_back((dep.pkg.clone(), depth + 1));

                    v.insert(idx);
                    idx
                }
            };

            let child_is_proc_macro = graph[child_idx].is_proc_macro;

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
                if skip_dep(config, info) {
                    continue;
                }

                graph.add_edge(
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

    Ok(graph)
}

fn get_package<'a>(packages: &'a [MetaPackage], pkg_id: &PackageId) -> &'a MetaPackage {
    packages.iter().find(|pkg| pkg.id == *pkg_id).unwrap()
}

pub(crate) fn skip_dep(config: &Config, info: &cargo_metadata::DepKindInfo) -> bool {
    (!config.build_deps && info.kind == MetaDepKind::Build)
        || (!config.dev_deps && info.kind == MetaDepKind::Development)
        || (!config.target_deps && info.target.is_some())
}
