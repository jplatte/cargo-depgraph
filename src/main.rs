use cargo_metadata::{DependencyKind, MetadataCommand};
use petgraph::dot::{Config, Dot};

// `DepInfo` represents the data associated with dependency graph edges
mod dep_info;
// `Package` represents the data associated with dependency graph nodes
mod package;

// Contains the `DepGraph` type and most of the graph building / analysis logic
mod graph;

use self::graph::{get_dep_graph, update_dep_info};

fn main() -> anyhow::Result<()> {
    let metadata = MetadataCommand::new().exec()?;

    let mut graph = get_dep_graph(metadata)?;
    update_dep_info(&mut graph);

    println!(
        "{:?}",
        Dot::with_attr_getters(
            &graph,
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
            }
        )
    );

    Ok(())
}

fn attr_for_dep_kind(kind: DependencyKind) -> Option<&'static str> {
    match kind {
        DependencyKind::Normal => None,
        DependencyKind::Development => Some("color = blue"),
        DependencyKind::Build => Some("color = darkgreen"),
        DependencyKind::Unknown => Some("color = red"),
    }
}
