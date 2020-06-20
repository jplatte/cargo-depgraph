use cargo_metadata::MetadataCommand;
use petgraph::dot::{Config, Dot};

mod graph;
mod package;

use self::graph::get_dep_graph;

fn main() -> anyhow::Result<()> {
    let metadata = MetadataCommand::new().exec()?;

    let graph = get_dep_graph(metadata)?;

    println!(
        "{:?}",
        Dot::with_attr_getters(
            &graph,
            &[Config::EdgeNoLabel],
            &|_, edge| {
                let dep = edge.weight();
                let mut attrs = Vec::new();

                if dep.dep_kinds.iter().all(|k| k.target.is_some()) {
                    attrs.push("color = red".to_owned());
                }

                attrs.join(", ")
            },
            &|_, (_, pkg)| {
                let mut attrs = Vec::new();

                if pkg.flags.is_ws_member {
                    attrs.push("shape = box".to_owned());
                }
                if pkg.flags.is_target_dep {
                    attrs.push("color = red".to_owned());
                }

                attrs.join(", ")
            }
        )
    );

    Ok(())
}
