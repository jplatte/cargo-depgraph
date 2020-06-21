use cargo_metadata::MetadataCommand;
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

                if dep.is_target_dep {
                    attrs.push("arrowType = empty".to_owned());
                    attrs.push("fillcolor = lightgrey".to_owned());
                }

                attrs.join(", ")
            },
            &|_, (_, pkg)| {
                let mut attrs = Vec::new();

                match pkg.dep_info {
                    Some(info) => {
                        if info.is_target_dep {
                            attrs.push("style = filled".to_owned());
                            attrs.push("fillcolor = lightgrey".to_owned());
                        }
                    }
                    None => {
                        // Workspace member
                        attrs.push("shape = box".to_owned());
                    }
                }

                attrs.join(", ")
            }
        )
    );

    Ok(())
}
