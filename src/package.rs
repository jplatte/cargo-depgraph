use std::fmt::{self, Debug, Formatter};

use cargo_metadata::Source;
use semver::Version;

use crate::dep_info::{DepInfo, DepKind};

#[derive(Clone)]
pub struct Package {
    pub name: String,
    pub version: Version,
    pub source: Option<Source>,
    pub dep_info: Option<DepInfo>,
}

impl Package {
    pub fn new(pkg: cargo_metadata::Package, is_ws_member: bool) -> Self {
        Self {
            name: pkg.name,
            version: pkg.version,
            source: pkg.source,
            dep_info: if is_ws_member { None } else { Some(DepInfo::default()) },
        }
    }

    pub fn dep_kind(&self) -> DepKind {
        self.dep_info.map(|di| di.kind).unwrap_or(DepKind::Normal)
    }
}

impl Debug for Package {
    // TODO: Allow writing version and such
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
