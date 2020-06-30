use petgraph::dot::{Config, Dot};

use crate::{dep_info::DepKind, graph::DepGraph};

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

            if dep.is_optional {
                attrs.push("style = dashed");
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

                    match (info.is_target_dep, info.is_optional) {
                        (true, true) => {
                            attrs.push("style = \"dashed,filled\"");
                            attrs.push("fillcolor = lightgrey");
                        }
                        (true, false) => {
                            attrs.push("style = filled");
                            attrs.push("fillcolor = lightgrey");
                        }
                        (false, true) => {
                            attrs.push("style = dashed");
                        }
                        (false, false) => {}
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

fn attr_for_dep_kind(kind: DepKind) -> Option<&'static str> {
    match kind {
        DepKind::Normal => None,
        DepKind::Dev => Some("color = blue"),
        DepKind::Build => Some("color = green3"),
        DepKind::BuildOfDev => Some("color = turquoise3"),
        DepKind::NormalAndBuild => Some("color = darkgreen"),
        DepKind::DevAndBuild => Some("color = darkviolet"),
        DepKind::NormalAndBuildOfDev => Some("color = turquoise4"),
        DepKind::DevAndBuildOfDev => Some("color = steelblue"),
        DepKind::Unknown => Some("color = red"),
    }
}
