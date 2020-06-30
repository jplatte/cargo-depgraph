use cargo_metadata::DependencyKind as MetaDepKind;

#[derive(Clone, Copy, Debug, Default)]
pub struct DepInfo {
    pub kind: DepKind,

    // TODO: instead collect targets, once we can actually evaluate whether they apply
    // (would be a really nice feature to show a linux- or windows-specific depgraph)
    pub is_target_dep: bool,

    /// whether this dependency could be removed by deactivating a cargo feature
    pub is_optional: bool,

    /// if optional, whether this is optional directly or transitively
    pub is_optional_direct: bool,

    /// whether this edge has been updated by update_dep_info after being inserted into the graph
    pub visited: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DepKind {
    pub host: BuildFlag,
    pub target: BuildFlag,
}

impl DepKind {
    pub const NORMAL: Self = Self { host: BuildFlag::Never, target: BuildFlag::Always };
    pub const BUILD: Self = Self { host: BuildFlag::Always, target: BuildFlag::Never };
    pub const DEV: Self = Self { host: BuildFlag::Never, target: BuildFlag::Test };

    pub const BUILD_OF_DEV: Self = Self { host: BuildFlag::Test, target: BuildFlag::Never };
    pub const NORMAL_AND_BUILD: Self = Self { host: BuildFlag::Always, target: BuildFlag::Always };
    pub const DEV_AND_BUILD: Self = Self { host: BuildFlag::Always, target: BuildFlag::Test };
    pub const NORMAL_AND_BUILD_OF_DEV: Self =
        Self { host: BuildFlag::Test, target: BuildFlag::Always };
    pub const DEV_AND_BUILD_OF_DEV: Self = Self { host: BuildFlag::Test, target: BuildFlag::Test };

    pub const UNKNOWN: Self = Self { host: BuildFlag::Never, target: BuildFlag::Never };

    pub fn combine_incoming(&mut self, other: Self) {
        *self = match (*self, other) {
            (Self::UNKNOWN, _) | (_, Self::UNKNOWN) => Self::UNKNOWN,

            (Self::NORMAL_AND_BUILD, _)
            | (_, Self::NORMAL_AND_BUILD)
            | (Self::NORMAL, Self::BUILD)
            | (Self::BUILD, Self::NORMAL)
            | (Self::NORMAL_AND_BUILD_OF_DEV, Self::BUILD | Self::DEV_AND_BUILD)
            | (Self::BUILD | Self::DEV_AND_BUILD, Self::NORMAL_AND_BUILD_OF_DEV) => {
                Self::NORMAL_AND_BUILD
            }

            (Self::NORMAL_AND_BUILD_OF_DEV, _)
            | (_, Self::NORMAL_AND_BUILD_OF_DEV)
            | (Self::NORMAL, Self::BUILD_OF_DEV)
            | (Self::BUILD_OF_DEV, Self::NORMAL) => Self::NORMAL_AND_BUILD_OF_DEV,

            (Self::DEV_AND_BUILD, _)
            | (_, Self::DEV_AND_BUILD)
            | (Self::DEV, Self::BUILD)
            | (Self::BUILD, Self::DEV) => Self::DEV_AND_BUILD,

            (Self::DEV_AND_BUILD_OF_DEV, _)
            | (_, Self::DEV_AND_BUILD_OF_DEV)
            | (Self::DEV, Self::BUILD_OF_DEV)
            | (Self::BUILD_OF_DEV, Self::DEV) => Self::DEV_AND_BUILD_OF_DEV,

            (Self::NORMAL, Self::NORMAL)
            | (Self::NORMAL, Self::DEV)
            | (Self::DEV, Self::NORMAL) => Self::NORMAL,

            (Self::BUILD, Self::BUILD)
            | (Self::BUILD, Self::BUILD_OF_DEV)
            | (Self::BUILD_OF_DEV, Self::BUILD) => Self::BUILD,

            (Self::DEV, Self::DEV) => Self::DEV,
            (Self::BUILD_OF_DEV, Self::BUILD_OF_DEV) => Self::BUILD_OF_DEV,
        };
    }

    pub fn update_outgoing(&mut self, node_kind: Self) {
        *self = match (node_kind, *self) {
            // don't update unknown outgoing edges
            (_, Self::UNKNOWN) => Self::UNKNOWN,

            // if node dep kind is unknown, keep the outgoing kind
            (Self::UNKNOWN, out) => out,

            // normal edges get the node kind
            (out, Self::NORMAL) => out,

            (Self::NORMAL_AND_BUILD, _) => Self::NORMAL_AND_BUILD,
            (Self::NORMAL_AND_BUILD_OF_DEV, _) => Self::NORMAL_AND_BUILD_OF_DEV,
            (Self::DEV_AND_BUILD, _) => Self::DEV_AND_BUILD,
            (Self::DEV_AND_BUILD_OF_DEV, _) => Self::DEV_AND_BUILD_OF_DEV,

            (Self::DEV, Self::BUILD | Self::BUILD_OF_DEV) => Self::BUILD_OF_DEV,

            (Self::BUILD_OF_DEV, Self::BUILD | Self::BUILD_OF_DEV) => Self::BUILD_OF_DEV,
            (Self::NORMAL | Self::BUILD, Self::BUILD) => Self::BUILD,

            // This function should never be called with dev dependencies, unless those got marked
            // as dev in an earlier update pass (hopefully we won't do multiple passes forever)
            (Self::DEV, Self::DEV) => Self::DEV,
            (n, Self::DEV) => {
                eprintln!("node {:?} has dev edge", n);
                Self::UNKNOWN
            }

            // These should just be impossible in general
            (Self::NORMAL | Self::BUILD, Self::BUILD_OF_DEV)
            | (
                Self::NORMAL | Self::BUILD | Self::DEV | Self::BUILD_OF_DEV,
                Self::NORMAL_AND_BUILD
                | Self::DEV_AND_BUILD
                | Self::NORMAL_AND_BUILD_OF_DEV
                | Self::DEV_AND_BUILD_OF_DEV,
            ) => {
                eprintln!("node {:?} has edge {:?}", node_kind, *self);
                Self::UNKNOWN
            }
        }
    }
}

impl Default for DepKind {
    fn default() -> Self {
        Self::from(MetaDepKind::Normal)
    }
}

impl From<MetaDepKind> for DepKind {
    fn from(kind: MetaDepKind) -> Self {
        match kind {
            MetaDepKind::Normal => Self::NORMAL,
            MetaDepKind::Build => Self::BUILD,
            MetaDepKind::Development => Self::DEV,
            MetaDepKind::Unknown => Self::UNKNOWN,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildFlag {
    Always,
    Test,
    Never,
}
