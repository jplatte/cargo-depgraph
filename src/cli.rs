use clap::{App, Arg, SubCommand};

pub struct Config {
    pub normal_deps: bool,
    pub build_deps: bool,
    pub dev_deps: bool,
    pub target_deps: bool,
    pub dedup_transitive_deps: bool,
}

pub fn parse_options() -> Config {
    let matches = App::new("cargo-depgraph")
        .bin_name("cargo")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            SubCommand::with_name("depgraph")
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
                .arg(
                    Arg::with_name("no_normal_deps")
                        .long("no-normal-deps")
                        .help("Don't include normal dependencies in the graph"),
                )
                .arg(Arg::with_name("dedup_transitive_deps").long("dedup-transitive-deps").help(
                    "Remove direct dependency edges where there's at \
                    least one transitive dependency of the same kind.",
                )),
        )
        .get_matches();

    let matches = matches.subcommand_matches("depgraph").unwrap();

    let all_deps = matches.is_present("all_deps");
    let normal_deps = !matches.is_present("no_normal_deps");
    let build_deps = all_deps || matches.is_present("build_deps");
    let dev_deps = all_deps || matches.is_present("dev_deps");
    let target_deps = all_deps || matches.is_present("target_deps");
    let dedup_transitive_deps = matches.is_present("dedup_transitive_deps");

    Config { normal_deps, build_deps, dev_deps, target_deps, dedup_transitive_deps }
}
