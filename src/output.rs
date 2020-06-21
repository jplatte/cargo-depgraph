use cargo_metadata::DependencyKind;
use petgraph::dot::{Config, Dot};

use crate::graph::DepGraph;

pub fn dot(graph: &DepGraph) -> Dot<'_, &DepGraph> {
    Dot::with_attr_getters(
        graph,
        &[Config::EdgeNoLabel],
        &|_, edge| {
            let dep = edge.weight();
            let mut attrs = Vec::new();

            if let Some(attr) = attr_for_dep_kind(dep.kind) {
                attrs.push(attr);
            }

            if dep.is_target_dep {
                attrs.push("arrowType = empty");
                attrs.push("fillcolor = lightgrey");
            }

            attrs.join(", ")
        },
        &|_, (_, pkg)| {
            let mut attrs = Vec::new();

            match pkg.dep_info {
                Some(info) => {
                    if let Some(attr) = attr_for_dep_kind(info.kind) {
                        attrs.push(attr);
                    }

                    if info.is_target_dep {
                        attrs.push("style = filled");
                        attrs.push("fillcolor = lightgrey");
                    }
                }
                None => {
                    // Workspace member
                    attrs.push("shape = box");
                }
            }

            attrs.join(", ")
        },
    )
}

fn attr_for_dep_kind(kind: DependencyKind) -> Option<&'static str> {
    match kind {
        DependencyKind::Normal => None,
        DependencyKind::Development => Some("color = blue"),
        DependencyKind::Build => Some("color = green3"),
        DependencyKind::Unknown => Some("color = red"),
    }
}
