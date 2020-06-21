use cargo_metadata::DependencyKind;

#[derive(Clone, Copy, Debug, Default)]
pub struct DepInfo {
    pub kind: DependencyKind,
    pub is_target_dep: bool,
}
