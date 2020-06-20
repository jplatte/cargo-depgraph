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
            &|_, _edge| { format!("") },
            &|_, node| {
                let pkg = node.1;
                let mut attrs = Vec::new();

                if pkg.is_ws_member {
                    attrs.push("shape = box".to_owned());
                }

                attrs.join(",")
            }
        )
    );

    Ok(())
}
