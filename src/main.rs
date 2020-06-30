use cargo_metadata::MetadataCommand;

// `DepInfo` represents the data associated with dependency graph edges
mod dep_info;
// `Package` represents the data associated with dependency graph nodes
mod package;

// Contains the `DepGraph` type and most of the graph building / analysis logic
mod graph;

// Command-line parsing
mod cli;
// Dot output generation
mod output;

use self::{
    cli::parse_options,
    graph::{dedup_transitive_deps, get_dep_graph, update_dep_info},
    output::dot,
};

fn main() -> anyhow::Result<()> {
    let config = parse_options();
    let metadata = MetadataCommand::new().exec()?;

    let mut graph = get_dep_graph(metadata, &config)?;
    update_dep_info(&mut graph);
    if config.dedup_transitive_deps {
        dedup_transitive_deps(&mut graph);
    }

    println!("{:?}", dot(&graph));

    Ok(())
}
