use std::fmt::{self, Debug, Formatter};

use cargo_metadata::{DepKindInfo, DependencyKind, NodeDep, Source};
use semver::Version;

#[derive(Clone)]
pub struct Package {
    pub name: String,
    pub version: Version,
    pub source: Option<Source>,
    pub flags: PackageFlags,
}

impl Package {
    pub fn new(pkg: cargo_metadata::Package, flags: PackageFlags) -> Self {
        Self { name: pkg.name, version: pkg.version, source: pkg.source, flags }
    }
}

impl Debug for Package {
    // TODO: Allow writing version and such
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Clone, Copy)]
pub struct PackageFlags {
    /// Whether this package belongs to the workspace
    pub is_ws_member: bool,
    /// Whether this dependency is always target-specific
    pub is_target_dep: bool,
    /// Whether this dependency is a dev dependency
    pub is_dev_dep: DepFlag,
}

impl PackageFlags {
    pub fn root() -> Self {
        Self { is_ws_member: true, is_target_dep: false, is_dev_dep: Never }
    }

    pub fn combine(self, other: Self) -> Self {
        Self {
            is_ws_member: self.is_ws_member || other.is_ws_member,
            is_target_dep: self.is_target_dep && other.is_target_dep,
            is_dev_dep: self.is_dev_dep.combine(other.is_dev_dep),
        }
    }
}

impl From<&DepKindInfo> for PackageFlags {
    fn from(info: &DepKindInfo) -> Self {
        Self {
            is_ws_member: false,
            is_target_dep: info.target.is_some(),
            is_dev_dep: (info.kind == DependencyKind::Development).into(),
        }
    }
}

impl From<&NodeDep> for PackageFlags {
    fn from(dep: &NodeDep) -> Self {
        let mut res = Self::from(&dep.dep_kinds[0]);
        for kind in &dep.dep_kinds[1..] {
            res = res.combine(kind.into());
        }
        res
    }
}

#[derive(Clone, Copy)]
pub enum DepFlag {
    Never,
    Sometimes,
    Always,
}

use DepFlag::*;

impl DepFlag {
    fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Never, Never) => Never,
            (Always, Always) => Always,
            _ => Sometimes,
        }
    }
}

impl From<bool> for DepFlag {
    fn from(b: bool) -> Self {
        if b {
            Always
        } else {
            Never
        }
    }
}
