use std::collections::VecDeque;

use petgraph::{
    algo::all_simple_paths,
    stable_graph::{NodeIndex, StableDiGraph},
    Direction,
};

use crate::{dep_info::DepInfo, package::Package};

mod build;

pub(crate) use build::get_dep_graph;

pub(crate) type DepGraph = StableDiGraph<Package, DepInfo, u16>;

pub(crate) fn update_dep_info(graph: &mut DepGraph) {
    for idx in graph.node_indices().collect::<Vec<_>>() {
        // We're only mutating nodes, not adding or deleting them, so we can safely use the indices
        // that were collected at the start throughout to visit each node once (or more than once,
        // in case we recurse inside update_node).
        update_node(graph, idx);
    }
}

fn update_node(graph: &mut DepGraph, idx: NodeIndex<u16>) {
    let is_ws_member = graph[idx].is_ws_member;

    let mut incoming = graph.neighbors_directed(idx, Direction::Incoming).detach();
    let mut node_info: Option<DepInfo> = None;
    while let Some((edge_idx, node_idx)) = incoming.next(graph) {
        // Don't backtrack on reverse dev-dependencies of workspace members
        let ws_reverse_dev_dep = is_ws_member && graph[edge_idx].kind.is_dev_only();

        if !ws_reverse_dev_dep && !graph[edge_idx].visited {
            update_node(graph, node_idx);
        }

        let edge_info = graph[edge_idx];

        if let Some(i) = &mut node_info {
            i.is_target_dep &= edge_info.is_target_dep;
            i.is_optional &= edge_info.is_optional;
            i.kind.combine_incoming(edge_info.kind);
        } else {
            node_info = Some(edge_info);
        }
    }

    let node_info = if is_ws_member {
        graph[idx].dep_info
    } else {
        let res = node_info.expect("non-workspace members to have at least one incoming edge");
        graph[idx].dep_info = res;
        res
    };

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

pub(crate) fn remove_irrelevant_deps(graph: &mut DepGraph, focus: &[String]) {
    let mut visit_queue: VecDeque<_> = graph.externals(Direction::Outgoing).collect();
    while let Some(idx) = visit_queue.pop_front() {
        // A node can end up being in the list multiple times. If it was already removed by a
        // previous iteration of this loop, skip it.
        if !graph.contains_node(idx) {
            continue;
        }

        let pkg = &graph[idx];
        if focus.contains(&pkg.name)
            || graph.neighbors_directed(idx, Direction::Outgoing).next().is_some()
        {
            // If the package is focused or has outgoing edges, don't remove it and continue with
            // the queue.
            continue;
        }

        // The package node at `idx` should be removed.
        // => First add its dependencies to the visit queue
        visit_queue.extend(graph.neighbors_directed(idx, Direction::Incoming));
        // => Then actually remove it
        graph.remove_node(idx);
    }
}

pub(crate) fn remove_deps(graph: &mut DepGraph, hide: &[String]) {
    let mut visit_queue: VecDeque<_> = graph.node_indices().collect();
    while let Some(idx) = visit_queue.pop_front() {
        // A node can end up being in the list multiple times. If it was already removed by a
        // previous iteration of this loop, skip it.
        if !graph.contains_node(idx) {
            continue;
        }

        let pkg = &graph[idx];

        let is_hidden = hide.contains(&pkg.name);

        if !is_hidden
            && (graph.neighbors_directed(idx, Direction::Incoming).next().is_some()
                || pkg.is_ws_member)
        {
            // If the package is not explicitly hidden, and also has incoming edges or is a
            // workspace members, don't remove it and continue with the queue.
            continue;
        }

        // The package node at `idx` should be removed.
        // => First add its dependencies to the visit queue
        visit_queue.extend(graph.neighbors_directed(idx, Direction::Outgoing));
        // => Then actually remove it
        graph.remove_node(idx);
    }
}

pub(crate) fn dedup_transitive_deps(graph: &mut DepGraph) {
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
        }
    }
}
