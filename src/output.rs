use base64::{prelude::BASE64_STANDARD, Engine};
use petgraph::dot::{Config, Dot};

use crate::{dep_info::DepKind, graph::DepGraph};

pub(crate) fn dot(graph: &DepGraph, is_html: bool) -> String {
    format!(
        "{:?}",
        Dot::with_attr_getters(
            graph,
            &[Config::EdgeNoLabel],
            &|_, edge| {
                let dep = edge.weight();
                let mut attrs = Vec::new();

                attrs.extend_from_slice(attr_for_dep_kind(dep.kind, is_html));

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

                attrs.extend_from_slice(attr_for_dep_kind(pkg.dep_info.kind, is_html));

                match (pkg.dep_info.is_target_dep, pkg.dep_info.is_optional) {
                    (true, true) => {
                        attrs.push("style = \"dashed,filled\"");
                        attrs.push("fillcolor = lightgrey");
                        attrs.push("fontcolor = black");
                    }
                    (true, false) => {
                        attrs.push("style = filled");
                        attrs.push("fillcolor = lightgrey");
                        attrs.push("fontcolor = black");
                    }
                    (false, true) => {
                        attrs.push("style = dashed");
                    }
                    (false, false) => {}
                }

                attrs.join(", ")
            },
        )
    )
}

pub(crate) fn html(graph: &DepGraph) -> String {
    let dot_base64 = BASE64_STANDARD.encode(dot(graph, true));

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

fn attr_for_dep_kind(kind: DepKind, is_html: bool) -> &'static [&'static str] {
    // The Dot parser of vizjs is not fully compatible with graphviz.
    // For example, color are applied to both background and border, some color names are not supported.
    // So this function manually sets the style for HTML output to avoid hurting eyes.
    if is_html {
        match kind {
            DepKind::NORMAL => &["color = \"#f5ecd5\""],
            DepKind::DEV => &["color = blue", "fontcolor = white"],
            DepKind::BUILD => &["color = \"#00cd00\"", "fontcolor = white"],
            DepKind::BUILD_OF_DEV => &["color = \"#00c5cd\""],
            DepKind::NORMAL_AND_BUILD => &["color = darkgreen", "fontcolor = white"],
            DepKind::DEV_AND_BUILD => &["color = darkviolet", "fontcolor = white"],
            DepKind::NORMAL_AND_BUILD_OF_DEV => &["color = \"#00868b\""],
            DepKind::DEV_AND_BUILD_OF_DEV => &["color = steelblue"],
            DepKind::UNKNOWN => &["color = red"],
        }
    } else {
        match kind {
            DepKind::NORMAL => &[],
            DepKind::DEV => &["color = blue"],
            DepKind::BUILD => &["color = green3"],
            DepKind::BUILD_OF_DEV => &["color = turquoise3"],
            DepKind::NORMAL_AND_BUILD => &["color = darkgreen"],
            DepKind::DEV_AND_BUILD => &["color = darkviolet"],
            DepKind::NORMAL_AND_BUILD_OF_DEV => &["color = turquoise4"],
            DepKind::DEV_AND_BUILD_OF_DEV => &["color = steelblue"],
            DepKind::UNKNOWN => &["color = red"],
        }
    }
}
