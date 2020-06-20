use std::fmt::{self, Debug, Display, Formatter};

use cargo_metadata::{PackageId, Source};
use semver::Version;

#[derive(Clone)]
pub struct Package {
    pub name: String,
    pub version: Version,
    pub source: Option<Source>,
    /// Whether this package belongs to the workspace
    pub is_ws_member: bool,
}

impl Package {
    pub fn new(pkg: cargo_metadata::Package, is_ws_member: bool) -> Self {
        Self { name: pkg.name, version: pkg.version, source: pkg.source, is_ws_member }
    }
}

impl Debug for Package {
    // TODO: Allow writing version and such
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
