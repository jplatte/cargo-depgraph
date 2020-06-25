# cargo-depgraph

Creates dependency graphs for cargo projects using `cargo metadata` and graphviz.

Usage: `cargo depgraph [options] | dot -Tpng > graph.png`

Commonly useful options:

* `--all-deps`

![cargo-depgraph's dependency graph](graph_all.png)

* `--all-deps --dedup-transitive-deps`

![cargo-depgraph's dependency graph with transitive dependency edges de-duplicated](graph_all_deduped.png)
