use std::{cell::Cell, collections::HashMap, rc::Rc};

use cargo_metadata::Package as MetaPackage;

use crate::graph::DepGraph;

pub(crate) fn set_name_stats(graph: &mut DepGraph) {
    let mut name_uses_map = HashMap::<String, Rc<Cell<u16>>>::new();
    for pkg in graph.node_weights_mut() {
        let name_uses = name_uses_map.entry(pkg.name.clone()).or_default().clone();
        name_uses.set(name_uses.get() + 1);

        pkg.name_uses = Some(name_uses);
    }
}

pub(crate) fn is_proc_macro(pkg: &MetaPackage) -> bool {
    let res = pkg.targets.iter().any(|t| t.kind.iter().any(|k| k == "proc-macro"));
    if res && pkg.targets.iter().any(|t| t.kind.iter().any(|k| k == "lib")) {
        eprintln!("encountered a crate that is both a regular library and a proc-macro");
    }

    res
}
