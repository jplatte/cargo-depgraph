use std::fmt::Display;

use base64::{prelude::BASE64_STANDARD, Engine};
use petgraph::dot::{Config, Dot};

use crate::{dep_info::DepKind, graph::DepGraph};

pub(crate) struct DotOutput<'a>(Dot<'a, &'a DepGraph>);

impl Display for DotOutput<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<'a> From<Dot<'a, &'a DepGraph>> for DotOutput<'a> {
    fn from(value: Dot<'a, &'a DepGraph>) -> Self {
        Self(value)
    }
}

pub(crate) fn dot(graph: &DepGraph) -> DotOutput<'_> {
    Dot::with_attr_getters(
        graph,
        &[Config::EdgeNoLabel],
        &|_, edge| {
            let dep = edge.weight();
            let mut attrs = Vec::new();

            if let Some(attr) = attr_for_dep_kind(dep.kind) {
                attrs.push(attr);
            }

            if dep.is_target_dep {
                attrs.push("arrowType = empty");
                attrs.push("fillcolor = lightgrey");
            }

            if dep.is_optional_direct {
                attrs.push("style = dotted");
            } else if dep.is_optional {
                attrs.push("style = dashed");
            }

            attrs.join(", ")
        },
        &|_, (_, pkg)| {
            let mut attrs = Vec::new();

            if pkg.is_ws_member {
                attrs.push("shape = box");
            }

            if let Some(attr) = attr_for_dep_kind(pkg.dep_info.kind) {
                attrs.push(attr);
            }

            match (pkg.dep_info.is_target_dep, pkg.dep_info.is_optional) {
                (true, true) => {
                    attrs.push("style = \"dashed,filled\"");
                    attrs.push("fillcolor = lightgrey");
                }
                (true, false) => {
                    attrs.push("style = filled");
                    attrs.push("fillcolor = lightgrey");
                }
                (false, true) => {
                    attrs.push("style = dashed");
                }
                (false, false) => {}
            }

            attrs.join(", ")
        },
    )
    .into()
}

pub(crate) fn html(graph: &DepGraph) -> String {
    let dot_base64 = BASE64_STANDARD.encode(dot(graph).to_string());

    const TEMPLATE: &str = r#"
<html>
<head>
    <script type="text/javascript" src="https://unpkg.com/vis-network@9.1.9/standalone/umd/vis-network.min.js"></script>

    <style type="text/css">
        #depgraph {
            width: 100%;
            height: 100%;
        }
    </style>
</head>
<body>
<div id="depgraph"></div>

<script type="text/javascript">
    let container = document.getElementById('depgraph');
    let dotGraph = atob("@BASE64_ENCODED_DOT@");
    let { nodes, edges, options } = vis.parseDOTNetwork(dotGraph);
    let network = new vis.Network(container, { nodes, edges }, options);
</script>
</body>
</html>
"#;
    TEMPLATE.replace("@BASE64_ENCODED_DOT@", &dot_base64)
}

fn attr_for_dep_kind(kind: DepKind) -> Option<&'static str> {
    match kind {
        DepKind::NORMAL => None,
        DepKind::DEV => Some("color = blue"),
        DepKind::BUILD => Some("color = green3"),
        DepKind::BUILD_OF_DEV => Some("color = turquoise3"),
        DepKind::NORMAL_AND_BUILD => Some("color = darkgreen"),
        DepKind::DEV_AND_BUILD => Some("color = darkviolet"),
        DepKind::NORMAL_AND_BUILD_OF_DEV => Some("color = turquoise4"),
        DepKind::DEV_AND_BUILD_OF_DEV => Some("color = steelblue"),
        DepKind::UNKNOWN => Some("color = red"),
    }
}
