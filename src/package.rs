use std::{
    cell::Cell,
    fmt::{self, Debug, Formatter},
    rc::Rc,
};

use cargo_metadata::{Package as MetaPackage, Source};
use semver::Version;

use crate::dep_info::{DepInfo, DepKind};

#[derive(Clone)]
pub struct Package {
    pub name: String,
    pub version: Version,
    pub source: Option<Source>,
    pub dep_info: DepInfo,
    pub is_ws_member: bool,

    pub name_uses: Option<Rc<Cell<u16>>>,
}

impl Package {
    pub fn new(pkg: &MetaPackage, is_ws_member: bool) -> Self {
        let mut dep_info = DepInfo::default();
        if is_proc_macro(pkg) {
            dep_info.kind = DepKind::BUILD;
        }

        Self {
            name: pkg.name.clone(),
            version: pkg.version.clone(),
            source: pkg.source.clone(),
            dep_info,
            is_ws_member,
            name_uses: None,
        }
    }
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

fn is_proc_macro(pkg: &MetaPackage) -> bool {
    let res = pkg.targets.iter().any(|t| t.kind.iter().any(|k| k == "proc-macro"));
    if res && pkg.targets.iter().any(|t| t.kind.iter().any(|k| k == "lib")) {
        eprintln!("enountered a crate that is both a regular library and a proc-macro");
    }

    res
}
