use clap::{App, AppSettings, Arg, SubCommand};

pub struct Config {
    pub build_deps: bool,
    pub dev_deps: bool,
    pub target_deps: bool,
    pub dedup_transitive_deps: bool,
    pub exclude: Vec<String>,
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

pub fn parse_options() -> Config {
    let matches = App::new("cargo-depgraph")
        .bin_name("cargo")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            SubCommand::with_name("depgraph")
                .settings(&[AppSettings::DeriveDisplayOrder, AppSettings::UnifiedHelpMessage])
                .arg(Arg::with_name("all_deps").long("all-deps").help(
                    "Include all dependencies in the graph \
                    (shorthand for --build-deps --dev-deps --target-deps)",
                ))
                .arg(
                    Arg::with_name("build_deps")
                        .long("build-deps")
                        .help("Include build-dependencies in the graph"),
                )
                .arg(
                    Arg::with_name("dev_deps")
                        .long("dev-deps")
                        .help("Include dev-dependencies in the graph"),
                )
                .arg(
                    Arg::with_name("target_deps")
                        .long("target-deps")
                        .help("Include cfg() dependencies in the graph"),
                )
                .arg(Arg::with_name("dedup_transitive_deps").long("dedup-transitive-deps").help(
                    "Remove direct dependency edges where there's at \
                    least one transitive dependency of the same kind.",
                ))
                .arg(
                    Arg::with_name("exclude")
                        .long("exclude")
                        .multiple(true)
                        .use_delimiter(true)
                        .help(
                            "Package name(s) to ignore; can be given as a \
                            comma-separated list or as multiple arguments",
                        ),
                )
                .arg(Arg::with_name("focus").long("focus").multiple(true).use_delimiter(true).help(
                    "Package name(s) to focus on: only the given packages, the workspace members \
                    that depend on them and any intermediate dependencies are going to be present \
                    in the output; can be given as a comma-separated list or as multiple arguments",
                ))
                // Options to pass through to `cargo metadata`
                .arg(
                    Arg::with_name("features")
                        .long("features")
                        .help("Space-separated list of features to activate")
                        .multiple(true)
                        .number_of_values(1)
                        .value_name("FEATURES"),
                )
                .arg(
                    Arg::with_name("all_features")
                        .long("all-features")
                        .help("Activate all available features"),
                )
                .arg(
                    Arg::with_name("no_default_features")
                        .long("no-default-features")
                        .help("Do not activate the `default` feature"),
                )
                .arg(
                    Arg::with_name("filter_platform")
                        .long("filter-platform")
                        .help("Only include resolve dependencies matching the given target-triple")
                        .multiple(true)
                        .number_of_values(1)
                        .value_name("TRIPLE"),
                )
                .arg(
                    Arg::with_name("manifest_path")
                        .long("manifest-path")
                        .help("Path to Cargo.toml")
                        .value_name("PATH"),
                )
                .arg(
                    Arg::with_name("frozen")
                        .long("frozen")
                        .help("Require Cargo.lock and cache are up to date"),
                )
                .arg(
                    Arg::with_name("locked")
                        .long("locked")
                        .help("Require Cargo.lock is up to date"),
                )
                .arg(
                    Arg::with_name("offline")
                        .long("offline")
                        .help("Run without accessing the network"),
                )
                .arg(
                    Arg::with_name("unstable_flags")
                        .short("Z")
                        .help(
                            "Unstable (nightly-only) flags to Cargo, see \
                            'cargo -Z help' for details",
                        )
                        .value_name("FLAG")
                        .multiple(true)
                        .number_of_values(1),
                ),
        )
        .get_matches();

    let matches = matches.subcommand_matches("depgraph").unwrap();

    let all_deps = matches.is_present("all_deps");
    let build_deps = all_deps || matches.is_present("build_deps");
    let dev_deps = all_deps || matches.is_present("dev_deps");
    let target_deps = all_deps || matches.is_present("target_deps");
    let dedup_transitive_deps = matches.is_present("dedup_transitive_deps");
    let exclude = matches.values_of("exclude").map_or_else(Vec::new, collect_owned);
    let focus = matches.values_of("focus").map_or_else(Vec::new, collect_owned);

    let features = matches.values_of("features").map_or_else(Vec::new, collect_owned);
    let all_features = matches.is_present("all_features");
    let no_default_features = matches.is_present("no_default_features");
    let filter_platform = matches.values_of("filter_platform").map_or_else(Vec::new, collect_owned);
    let manifest_path = matches.value_of("manifest_path").map(ToOwned::to_owned);
    let frozen = matches.is_present("frozen");
    let locked = matches.is_present("locked");
    let offline = matches.is_present("offline");
    let unstable_flags = matches.values_of("unstable_flags").map_or_else(Vec::new, collect_owned);

    Config {
        build_deps,
        dev_deps,
        target_deps,
        dedup_transitive_deps,
        exclude,
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

fn collect_owned<'a, T>(iter: impl Iterator<Item = &'a T>) -> Vec<T::Owned>
where
    T: ?Sized + ToOwned + 'a,
{
    iter.map(ToOwned::to_owned).collect()
}
