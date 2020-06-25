use cargo_metadata::DependencyKind as MetaDepKind;

#[derive(Clone, Copy, Debug, Default)]
pub struct DepInfo {
    pub kind: DepKind,
    // TODO: instead collect targets, once we can actually evaluate whether they apply
    // (would be a really nice feature to show a linux- or windows-specific depgraph)
    pub is_target_dep: bool,

    /// whether this edge has been updated by update_dep_info after being inserted into the graph
    pub visited: bool,
}

// TODO: potentially collapse this into sth like
// struct DepKind { host: BuildFlag, target: BuildFlag }
// enum BuildFlag { Always, Test, Never }
// (unknown could be represented by { host: Never, target: Never })
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DepKind {
    /// normal dep
    ///
    /// always built for target
    Normal,
    /// build dep
    ///
    /// always built for host
    Build,
    /// development dep (only compiled for tests)
    ///
    /// built for target in test mode
    Dev,
    /// build dep of development dep (only compiled for tests)
    ///
    /// built for host in test mode
    BuildOfDev,
    /// both normal dep and build dep (compiled twice if host != target or resolver v2 & normal dep
    /// and build dep have different features activated)
    ///
    /// always built for target and host
    NormalAndBuild,
    /// both development dep and build dep (compiled twice for tests if host != target or resolver
    /// v2 & dev dep and build dep have different features activated)
    ///
    /// always build for host, built for target in test mode
    DevAndBuild,
    /// both normal dep and build dep of development dep (compiled twice for tests if host != target
    /// or resolver v2 & normal dep and build-of-dev dep have different features activated)
    ///
    /// always built for target, built for host in test mode
    NormalAndBuildOfDev,
    /// both development dep and build dep of development dep (compiled twice for tests if host !=
    /// target or resolver v2 & dev dep and build-of-dev dep have different features activated)
    ///
    /// built for target and host in test mode
    DevAndBuildOfDev,
    /// unknown?
    Unknown,
}

impl DepKind {
    pub fn combine_incoming(&mut self, other: Self) {
        *self = match (*self, other) {
            (DepKind::Unknown, _) | (_, DepKind::Unknown) => DepKind::Unknown,

            (DepKind::NormalAndBuild, _)
            | (_, DepKind::NormalAndBuild)
            | (DepKind::Normal, DepKind::Build)
            | (DepKind::Build, DepKind::Normal)
            | (DepKind::NormalAndBuildOfDev, DepKind::Build | DepKind::DevAndBuild)
            | (DepKind::Build | DepKind::DevAndBuild, DepKind::NormalAndBuildOfDev) => {
                DepKind::NormalAndBuild
            }

            (DepKind::NormalAndBuildOfDev, _)
            | (_, DepKind::NormalAndBuildOfDev)
            | (DepKind::Normal, DepKind::BuildOfDev)
            | (DepKind::BuildOfDev, DepKind::Normal) => DepKind::NormalAndBuildOfDev,

            (DepKind::DevAndBuild, _)
            | (_, DepKind::DevAndBuild)
            | (DepKind::Dev, DepKind::Build)
            | (DepKind::Build, DepKind::Dev) => DepKind::DevAndBuild,

            (DepKind::DevAndBuildOfDev, _)
            | (_, DepKind::DevAndBuildOfDev)
            | (DepKind::Dev, DepKind::BuildOfDev)
            | (DepKind::BuildOfDev, DepKind::Dev) => DepKind::DevAndBuildOfDev,

            (DepKind::Normal, DepKind::Normal)
            | (DepKind::Normal, DepKind::Dev)
            | (DepKind::Dev, DepKind::Normal) => DepKind::Normal,

            (DepKind::Build, DepKind::Build)
            | (DepKind::Build, DepKind::BuildOfDev)
            | (DepKind::BuildOfDev, DepKind::Build) => DepKind::Build,

            (DepKind::Dev, DepKind::Dev) => DepKind::Dev,
            (DepKind::BuildOfDev, DepKind::BuildOfDev) => DepKind::BuildOfDev,
        };
    }

    pub fn update_outgoing(&mut self, node_kind: Self) {
        *self = match (node_kind, *self) {
            // don't update unknown outgoing edges
            (_, DepKind::Unknown) => DepKind::Unknown,

            // if node dep kind is unknown, keep the outgoing kind
            (DepKind::Unknown, out) => out,

            // normal edges get the node kind
            (out, DepKind::Normal) => out,

            (DepKind::NormalAndBuild, _) => DepKind::NormalAndBuild,
            (DepKind::NormalAndBuildOfDev, _) => DepKind::NormalAndBuildOfDev,
            (DepKind::DevAndBuild, _) => DepKind::DevAndBuild,
            (DepKind::DevAndBuildOfDev, _) => DepKind::DevAndBuildOfDev,

            (DepKind::Dev, DepKind::Build | DepKind::BuildOfDev) => DepKind::BuildOfDev,

            (DepKind::BuildOfDev, DepKind::Build | DepKind::BuildOfDev) => DepKind::BuildOfDev,
            (DepKind::Normal | DepKind::Build, DepKind::Build) => DepKind::Build,

            // This function should never be called with dev dependencies, unless those got marked
            // as dev in an earlier update pass (hopefully we won't do multiple passes forever)
            (DepKind::Dev, DepKind::Dev) => DepKind::Dev,
            (n, DepKind::Dev) => {
                eprintln!("node {:?} has dev edge", n);
                DepKind::Unknown
            }

            // These should just be impossible in general
            (DepKind::Normal | DepKind::Build, DepKind::BuildOfDev)
            | (
                DepKind::Normal | DepKind::Build | DepKind::Dev | DepKind::BuildOfDev,
                DepKind::NormalAndBuild
                | DepKind::DevAndBuild
                | DepKind::NormalAndBuildOfDev
                | DepKind::DevAndBuildOfDev,
            ) => {
                eprintln!("node {:?} has edge {:?}", node_kind, *self);
                DepKind::Unknown
            }
        };
    }
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
