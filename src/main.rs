use std::iter;

use cargo_metadata::MetadataCommand;

// `DepInfo` represents the data associated with dependency graph edges
mod dep_info;
// `Package` represents the data associated with dependency graph nodes
mod package;

// Contains the `DepGraph` type and most of the graph building / analysis logic
mod graph;
// Contains some auxiliary logic (currently just checking for packages of the same name)
mod util;

// Command-line parsing
mod cli;
// Dot output generation
mod output;

use self::{
    cli::parse_options,
    graph::{
        dedup_transitive_deps, get_dep_graph, remove_deps, remove_irrelevant_deps,
        remove_non_workspace_deps, update_dep_info,
    },
    output::dot,
    util::set_name_stats,
};

fn main() -> anyhow::Result<()> {
    let config = parse_options();
    let mut cmd = MetadataCommand::new();

    if let Some(path) = &config.manifest_path {
        cmd.manifest_path(path);
    }

    let mut other_options = Vec::new();
    other_options.extend(config.features.iter().flat_map(|f| cli_args("--features", f)));
    if config.all_features {
        other_options.push("--all-features".into());
    }
    if config.no_default_features {
        other_options.push("--no-default-features".into());
    }
    other_options
        .extend(config.filter_platform.iter().flat_map(|p| cli_args("--filter-platform", p)));
    if config.frozen {
        other_options.push("--frozen".into());
    }
    if config.locked {
        other_options.push("--locked".into());
    }
    if config.offline {
        other_options.push("--offline".into());
    }
    other_options.extend(config.unstable_flags.iter().flat_map(|f| cli_args("-Z", f)));

    let metadata = cmd.other_options(other_options).exec()?;

    let mut graph = get_dep_graph(metadata, &config)?;
    update_dep_info(&mut graph);
    if config.workspace_only {
        remove_non_workspace_deps(&mut graph);
    }
    if !config.focus.is_empty() {
        remove_irrelevant_deps(&mut graph, &config.focus);
    }
    if !config.hide.is_empty() {
        remove_deps(&mut graph, &config.hide);
    }
    if config.dedup_transitive_deps {
        dedup_transitive_deps(&mut graph);
    }
    set_name_stats(&mut graph);

    println!("{:?}", dot(&graph));

    Ok(())
}

fn cli_args(opt_name: &str, val: &str) -> impl Iterator<Item = String> {
    iter::once(opt_name.into()).chain(iter::once(val.into()))
}
