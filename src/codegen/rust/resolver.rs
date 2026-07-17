use crate::types::{DbType, RustType, TypeRegistry};

/// Resolves [`DbType`] to Rust type expressions using a configured
/// [`TypeRegistry`].
///
/// This replaces the old `dbtype_to_rust()` free function which silently
/// used `TypeRegistry::default()` and ignored user-configured type overrides.
pub(super) struct RustTypeResolver {
    registry: TypeRegistry,
}

impl RustTypeResolver {
    pub(super) fn new(registry: TypeRegistry) -> Self {
        Self { registry }
    }

    /// Resolve a [`DbType`] to its Rust type name, wrapping in `Option<T>`
    /// when `nullable` is `true`.
    pub(super) fn resolve(&self, ty: &DbType, nullable: bool) -> String {
        let rt = self.resolve_full(ty);
        if nullable {
            format!("Option<{}>", rt.name)
        } else {
            rt.name
        }
    }

    /// Resolve a [`DbType`] to the full [`RustType`] (name + imports).
    pub(super) fn resolve_full(&self, ty: &DbType) -> RustType {
        self.registry.resolve(ty)
    }
}
