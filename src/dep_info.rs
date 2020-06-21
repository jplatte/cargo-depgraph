use cargo_metadata::DependencyKind;

#[derive(Clone, Copy, Debug, Default)]
pub struct DepInfo {
    pub kind: DependencyKind,
    // TODO: instead collect targets, once we can actually evaluate whether they apply
    // (would be a really nice feature to show a linux- or windows-specific depgraph)
    pub is_target_dep: bool,
}
