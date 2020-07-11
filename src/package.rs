use std::{
    cell::Cell,
    fmt::{self, Debug, Formatter},
    rc::Rc,
};

use cargo_metadata::Source;
use semver::Version;

use crate::dep_info::DepInfo;

#[derive(Clone)]
pub struct Package {
    pub name: String,
    pub version: Version,
    pub source: Option<Source>,
    pub dep_info: Option<DepInfo>,

    pub name_uses: Option<Rc<Cell<u16>>>,
}

impl Package {
    pub fn new(pkg: &cargo_metadata::Package, is_ws_member: bool) -> Self {
        Self {
            name: pkg.name.clone(),
            version: pkg.version.clone(),
            source: pkg.source.clone(),
            dep_info: if is_ws_member { None } else { Some(DepInfo::default()) },
            name_uses: None,
        }
    }

    pub fn is_root(&self) -> bool {
        self.dep_info.is_none()
    }

    //pub fn dep_kind(&self) -> DepKind {
    //    self.dep_info.map(|di| di.kind).unwrap_or(DepKind::Normal)
    //}
}

impl Debug for Package {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if self.name_uses.as_ref().unwrap().get() > 1 {
            write!(f, " {}", self.version)?;
        }

        Ok(())
    }
}
