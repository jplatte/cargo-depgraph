use std::{
    cell::Cell,
    fmt::{self, Debug, Formatter},
    rc::Rc,
};

use cargo_metadata::{semver::Version, Package as MetaPackage};

use crate::{
    dep_info::{DepInfo, DepKind},
    util::is_proc_macro,
};

#[derive(Clone)]
pub(crate) struct Package {
    pub name: String,
    pub version: Version,
    pub dep_info: DepInfo,
    pub is_ws_member: bool,
    pub is_proc_macro: bool,

    pub name_uses: Option<Rc<Cell<u16>>>,
}

impl Package {
    pub fn new(pkg: &MetaPackage, is_ws_member: bool) -> Self {
        let mut dep_info = DepInfo::default();
        let is_proc_macro = is_proc_macro(pkg);
        if is_proc_macro {
            dep_info.kind = DepKind::BUILD;
        }

        Self {
            name: pkg.name.clone(),
            version: pkg.version.clone(),
            dep_info,
            is_ws_member,
            is_proc_macro,
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
