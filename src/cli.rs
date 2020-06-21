pub struct Config {
    pub normal_deps: bool,
    pub build_deps: bool,
    pub dev_deps: bool,
    pub target_deps: bool,
}

pub fn parse_options() -> Config {
    // TODO

    Config { normal_deps: true, build_deps: false, dev_deps: false, target_deps: false }
}
