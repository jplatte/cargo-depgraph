use clap::{Arg, ArgAction, Command};

pub(crate) struct Config {
    pub build_deps: bool,
    pub dev_deps: bool,
    pub target_deps: bool,
    pub dedup_transitive_deps: bool,
    pub hide: Vec<String>,
    pub exclude: Vec<String>,
    pub workspace_only: bool,
    pub focus: Vec<String>,

    pub features: Vec<String>,
    pub all_features: bool,
    pub no_default_features: bool,
    pub filter_platform: Vec<String>,
    pub manifest_path: Option<String>,
    pub frozen: bool,
    pub locked: bool,
    pub offline: bool,
    pub unstable_flags: Vec<String>,
}

pub(crate) fn parse_options() -> Config {
    let matches = Command::new("cargo-depgraph")
        .bin_name("cargo")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            Command::new("depgraph")
                .arg(Arg::new("all_deps").long("all-deps").action(ArgAction::SetTrue).help(
                    "Include all dependencies in the graph \
                     (shorthand for --build-deps --dev-deps --target-deps)",
                ))
                .arg(
                    Arg::new("build_deps")
                        .long("build-deps")
                        .action(ArgAction::SetTrue)
                        .help("Include build-dependencies in the graph"),
                )
                .arg(
                    Arg::new("dev_deps")
                        .long("dev-deps")
                        .action(ArgAction::SetTrue)
                        .help("Include dev-dependencies in the graph"),
                )
                .arg(
                    Arg::new("target_deps")
                        .long("target-deps")
                        .action(ArgAction::SetTrue)
                        .help("Include cfg() dependencies in the graph"),
                )
                .arg(
                    Arg::new("dedup_transitive_deps")
                        .long("dedup-transitive-deps")
                        .action(ArgAction::SetTrue)
                        .help(
                            "Remove direct dependency edges where there's at \
                             least one transitive dependency of the same kind.",
                        ),
                )
                .arg(
                    Arg::new("hide")
                        .long("hide")
                        .action(ArgAction::Append)
                        .value_delimiter(',')
                        .help(
                            "Package name(s) to hide; can be given as a comma-separated list or \
                             as multiple arguments\n\n\
                             In contrast to --exclude, hidden packages will still contribute in \
                             dependency kind resolution.",
                        ),
                )
                .arg(
                    Arg::new("exclude")
                        .long("exclude")
                        .action(ArgAction::Append)
                        .value_delimiter(',')
                        .help(
                            "Package name(s) to ignore; can be given as a comma-separated list or \
                             as multiple arguments\n\n\
                             In constrast to --hide, excluded packages will not contribute in \
                             dependency kind resolution",
                        ),
                )
                .arg(
                    Arg::new("workspace_only")
                        .long("workspace-only")
                        .action(ArgAction::SetTrue)
                        .help("Exclude all packages outside of the workspace"),
                )
                .arg(
                    Arg::new("focus")
                        .long("focus")
                        .action(ArgAction::Append)
                        .value_delimiter(',')
                        .help(
                            "Package name(s) to focus on: only the given packages, the workspace \
                             members that depend on them and any intermediate dependencies are \
                             going to be present in the output; can be given as a comma-separated \
                             list or as multiple arguments",
                        ),
                )
                // Options to pass through to `cargo metadata`
                .arg(
                    Arg::new("features")
                        .long("features")
                        .help("List of features to activate")
                        .action(ArgAction::Append)
                        .value_name("FEATURES"),
                )
                .arg(
                    Arg::new("all_features")
                        .long("all-features")
                        .action(ArgAction::SetTrue)
                        .help("Activate all available features"),
                )
                .arg(
                    Arg::new("no_default_features")
                        .long("no-default-features")
                        .action(ArgAction::SetTrue)
                        .help("Do not activate the `default` feature"),
                )
                .arg(
                    Arg::new("filter_platform")
                        .long("filter-platform")
                        .help("Only include resolve dependencies matching the given target-triple")
                        .action(ArgAction::Append)
                        .number_of_values(1)
                        .value_name("TRIPLE"),
                )
                .arg(
                    Arg::new("manifest_path")
                        .long("manifest-path")
                        .help("Path to Cargo.toml")
                        .value_name("PATH"),
                )
                .arg(
                    Arg::new("frozen")
                        .long("frozen")
                        .action(ArgAction::SetTrue)
                        .help("Require Cargo.lock and cache are up to date"),
                )
                .arg(
                    Arg::new("locked")
                        .long("locked")
                        .action(ArgAction::SetTrue)
                        .help("Require Cargo.lock is up to date"),
                )
                .arg(
                    Arg::new("offline")
                        .long("offline")
                        .action(ArgAction::SetTrue)
                        .help("Run without accessing the network"),
                )
                .arg(
                    Arg::new("unstable_flags")
                        .short('Z')
                        .help(
                            "Unstable (nightly-only) flags to Cargo, see \
                            'cargo -Z help' for details",
                        )
                        .value_name("FLAG")
                        .action(ArgAction::Append)
                        .number_of_values(1),
                ),
        )
        .get_matches();

    let matches = matches.subcommand_matches("depgraph").unwrap();

    let all_deps = matches.get_flag("all_deps");
    let build_deps = all_deps || matches.get_flag("build_deps");
    let dev_deps = all_deps || matches.get_flag("dev_deps");
    let target_deps = all_deps || matches.get_flag("target_deps");
    let dedup_transitive_deps = matches.get_flag("dedup_transitive_deps");
    let hide = matches.get_many("hide").map_or_else(Vec::new, collect_owned);
    let exclude = matches.get_many("exclude").map_or_else(Vec::new, collect_owned);
    let workspace_only = matches.get_flag("workspace_only");
    let focus = matches.get_many("focus").map_or_else(Vec::new, collect_owned);

    let features = matches.get_many("features").map_or_else(Vec::new, collect_owned);
    let all_features = matches.get_flag("all_features");
    let no_default_features = matches.get_flag("no_default_features");
    let filter_platform = matches.get_many("filter_platform").map_or_else(Vec::new, collect_owned);
    let manifest_path = matches.get_one("manifest_path").cloned();
    let frozen = matches.get_flag("frozen");
    let locked = matches.get_flag("locked");
    let offline = matches.get_flag("offline");
    let unstable_flags = matches.get_many("unstable_flags").map_or_else(Vec::new, collect_owned);

    Config {
        build_deps,
        dev_deps,
        target_deps,
        dedup_transitive_deps,
        hide,
        exclude,
        workspace_only,
        focus,
        features,
        all_features,
        no_default_features,
        filter_platform,
        manifest_path,
        frozen,
        locked,
        offline,
        unstable_flags,
    }
}

fn collect_owned<'a, T>(iter: impl Iterator<Item = &'a T>) -> Vec<T>
where
    T: ?Sized + Clone + 'a,
{
    iter.cloned().collect()
}
