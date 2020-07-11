use std::{cell::Cell, collections::HashMap, rc::Rc};

use crate::graph::DepGraph;

pub fn set_name_stats(graph: &mut DepGraph) {
    let mut name_uses_map = HashMap::<String, Rc<Cell<u16>>>::new();
    for pkg in graph.node_weights_mut() {
        let name_uses = name_uses_map.entry(pkg.name.clone()).or_default().clone();
        name_uses.set(name_uses.get() + 1);

        pkg.name_uses = Some(name_uses);
    }
}
