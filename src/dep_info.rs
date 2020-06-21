use cargo_metadata::DependencyKind as MetaDepKind;

#[derive(Clone, Copy, Debug, Default)]
pub struct DepInfo {
    pub kind: DepKind,
    // TODO: instead collect targets, once we can actually evaluate whether they apply
    // (would be a really nice feature to show a linux- or windows-specific depgraph)
    pub is_target_dep: bool,
}

// TODO: BuildOfDev, a build-dependency of a dev-dependency, which in contrast to BuildAndDev only
// ever runs on the host, never on the target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DepKind {
    Normal,
    Build,
    Dev,
    BuildAndDev,
    Unknown,
}

impl Default for DepKind {
    fn default() -> Self {
        DepKind::Normal
    }
}

impl From<MetaDepKind> for DepKind {
    fn from(kind: MetaDepKind) -> Self {
        match kind {
            MetaDepKind::Normal => Self::Normal,
            MetaDepKind::Build => Self::Build,
            MetaDepKind::Development => Self::Dev,
            MetaDepKind::Unknown => Self::Unknown,
        }
    }
}
