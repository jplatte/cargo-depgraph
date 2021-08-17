use clap::{App, AppSettings, Arg};

pub struct Config {
    pub build_deps: bool,
    pub dev_deps: bool,
    pub target_deps: bool,
    pub dedup_transitive_deps: bool,
    pub hide: Vec<String>,
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
            App::new("depgraph")
                .setting(AppSettings::DeriveDisplayOrder)
                .setting(AppSettings::UnifiedHelpMessage)
                .arg(Arg::new("all_deps").long("all-deps").about(
                    "Include all dependencies in the graph \
                     (shorthand for --build-deps --dev-deps --target-deps)",
                ))
                .arg(
                    Arg::new("build_deps")
                        .long("build-deps")
                        .about("Include build-dependencies in the graph"),
                )
                .arg(
                    Arg::new("dev_deps")
                        .long("dev-deps")
                        .about("Include dev-dependencies in the graph"),
                )
                .arg(
                    Arg::new("target_deps")
                        .long("target-deps")
                        .about("Include cfg() dependencies in the graph"),
                )
                .arg(Arg::new("dedup_transitive_deps").long("dedup-transitive-deps").about(
                    "Remove direct dependency edges where there's at \
                     least one transitive dependency of the same kind.",
                ))
                .arg(
                    Arg::new("hide")
                        .long("hide")
                        .multiple_occurrences(true)
                        .multiple_values(true)
                        .use_delimiter(true)
                        .about(
                            "Package name(s) to hide; can be given as a comma-separated list or \
                             as multiple arguments\n\n\
                             In contrast to --exclude, hidden packages will still contribute in \
                             dependency kind resolution.",
                        ),
                )
                .arg(
                    Arg::new("exclude")
                        .long("exclude")
                        .multiple_occurrences(true)
                        .multiple_values(true)
                        .use_delimiter(true)
                        .about(
                            "Package name(s) to ignore; can be given as a comma-separated list or \
                             as multiple arguments\n\n\
                             In constrast to --hide, excluded packages will not contribute in \
                             dependency kind resolution",
                        ),
                )
                .arg(
                    Arg::new("focus")
                        .long("focus")
                        .multiple_occurrences(true)
                        .multiple_values(true)
                        .use_delimiter(true)
                        .about(
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
                        .about("Space-separated list of features to activate")
                        .multiple_occurrences(true)
                        .multiple_values(true)
                        .number_of_values(1)
                        .value_name("FEATURES"),
                )
                .arg(
                    Arg::new("all_features")
                        .long("all-features")
                        .about("Activate all available features"),
                )
                .arg(
                    Arg::new("no_default_features")
                        .long("no-default-features")
                        .about("Do not activate the `default` feature"),
                )
                .arg(
                    Arg::new("filter_platform")
                        .long("filter-platform")
                        .about("Only include resolve dependencies matching the given target-triple")
                        .multiple_occurrences(true)
                        .multiple_values(true)
                        .number_of_values(1)
                        .value_name("TRIPLE"),
                )
                .arg(
                    Arg::new("manifest_path")
                        .long("manifest-path")
                        .about("Path to Cargo.toml")
                        .value_name("PATH"),
                )
                .arg(
                    Arg::new("frozen")
                        .long("frozen")
                        .about("Require Cargo.lock and cache are up to date"),
                )
                .arg(Arg::new("locked").long("locked").about("Require Cargo.lock is up to date"))
                .arg(Arg::new("offline").long("offline").about("Run without accessing the network"))
                .arg(
                    Arg::new("unstable_flags")
                        .short('Z')
                        .about(
                            "Unstable (nightly-only) flags to Cargo, see \
                            'cargo -Z help' for details",
                        )
                        .value_name("FLAG")
                        .multiple_occurrences(true)
                        .multiple_values(true)
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
    let hide = matches.values_of("hide").map_or_else(Vec::new, collect_owned);
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
        hide,
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
