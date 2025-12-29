use std::{
    collections::BTreeMap,
    sync::{LazyLock, RwLock},
};

use iref::Uri;
use uuid::Uuid;

use crate::{ConventionId, ZarrConventionImpl, convention::ConventionDefinition};

pub static DEFAULT_ZARR_CONVENTION_REGISTRY: LazyLock<ConventionRegistry> =
    LazyLock::new(Default::default);

#[derive(Debug, Default)]
pub struct ConventionRegistry {
    inner: RwLock<ConventionRegistryInner>,
}

/// All value [Convention]s will be fully populated.
#[derive(Debug, Clone, Default)]
struct ConventionRegistryInner {
    /// Keyed by UUID.
    uuid_reg: BTreeMap<Uuid, ConventionDefinition>,
    /// Keyed by schema URL.
    schema_reg: BTreeMap<&'static Uri, ConventionDefinition>,
    /// Keyed by spec URL.
    spec_reg: BTreeMap<&'static Uri, ConventionDefinition>,
}

impl ConventionRegistry {
    /// Register a given convention in this registry.
    ///
    /// ## Example
    ///
    /// ```
    /// use zarrs_conventions::{uuid, iref};
    /// use zarrs_conventions::{ZarrConventionImpl, ConventionDefinition, registry::ConventionRegistry};
    ///
    /// #[derive(serde::Serialize, serde::Deserialize)]
    /// pub struct MyConvention {
    ///     foo: String
    /// };
    ///
    /// impl ZarrConventionImpl for MyConvention {
    ///    const DEFINITION: ConventionDefinition = ConventionDefinition {
    ///        uuid: uuid::uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"),
    ///        schema_url: iref::uri!("https://example.com/schemas/my_convention.json"),
    ///        spec_url: iref::uri!("https://example.com/specs/my_convention"),
    ///        name: "my_convention",
    ///        description: "An example convention.",
    ///    };
    /// }
    ///
    /// let registry = ConventionRegistry::default();
    /// registry.register::<MyConvention>().unwrap();
    /// ```
    pub fn register<T: ZarrConventionImpl>(&self) -> Result<&Self, String> {
        let mut inner = self.inner.write().expect("RwLock poisoned");
        if inner
            .uuid_reg
            .insert(T::DEFINITION.uuid, T::DEFINITION)
            .is_some()
        {
            return Err(format!(
                "Convention with UUID {} is already registered",
                T::DEFINITION.uuid
            ));
        }
        if inner
            .schema_reg
            .insert(T::DEFINITION.schema_url, T::DEFINITION)
            .is_some()
        {
            return Err(format!(
                "Convention with schema URL {} is already registered",
                T::DEFINITION.schema_url
            ));
        }
        if inner
            .spec_reg
            .insert(T::DEFINITION.spec_url, T::DEFINITION)
            .is_some()
        {
            return Err(format!(
                "Convention with spec URL {} is already registered",
                T::DEFINITION.spec_url
            ));
        }
        Ok(self)
    }

    pub fn conventions(&self) -> Vec<ConventionDefinition> {
        let inner = self.inner.read().expect("RwLock poisoned");
        inner.uuid_reg.values().cloned().collect()
    }

    pub fn contains(&self, id: &ConventionId) -> bool {
        let inner = self.inner.read().expect("RwLock poisoned");
        match id {
            ConventionId::Uuid(uuid) => inner.uuid_reg.contains_key(uuid),
            ConventionId::SchemaUrl(url) => inner.schema_reg.contains_key(&url.as_ref()),
            ConventionId::SpecUrl(url) => inner.spec_reg.contains_key(&url.as_ref()),
        }
    }

    pub fn get(&self, id: &ConventionId) -> Option<ConventionDefinition> {
        let inner = self.inner.read().expect("RwLock poisoned");
        match id {
            ConventionId::Uuid(uuid) => inner.uuid_reg.get(uuid).copied(),
            ConventionId::SchemaUrl(url) => inner.schema_reg.get(&url.as_ref()).copied(),
            ConventionId::SpecUrl(url) => inner.spec_reg.get(&url.as_ref()).copied(),
        }
    }
}

/// Register conventions in the default registry.
/// Multiple conventions can be registered at once.
/// This macro can only be called once per module.
///
/// Panics if registration fails (for example, due to duplicate identifiers).
///
/// ## Example
///
/// ```
/// use zarrs_conventions::{uuid, iref};
/// use zarrs_conventions::{DEFAULT_ZARR_CONVENTION_REGISTRY, ZarrConventionImpl, ConventionDefinition, register_zarr_conventions};
///
/// #[derive(serde::Serialize, serde::Deserialize)]
/// pub struct MyConvention {foo: String};
///
/// impl ZarrConventionImpl for MyConvention {
///    const DEFINITION: ConventionDefinition = ConventionDefinition {
///        uuid: uuid::uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"),
///        schema_url: iref::uri!("https://example.com/schemas/my_convention.json"),
///        spec_url: iref::uri!("https://example.com/specs/my_convention"),
///        name: "my_convention",
///        description: "An example convention.",
///    };
/// }
///
/// register_zarr_conventions!(MyConvention);
/// ```
#[macro_export]
macro_rules! register_zarr_conventions {
    ($($convention:ty),+) => {
        $(
            #[ctor::ctor]
            fn register_convention() {
                $crate::DEFAULT_ZARR_CONVENTION_REGISTRY.register::<$convention>().map_err(|e|
                    panic!("Failed to register convention {}: {}", stringify!($convention), e)
                );
            }
        )+
    };
}

#[cfg(test)]
mod tests {
    use iref::uri;

    use crate::{
        ZarrConventionImpl, convention::ConventionDefinition, registry::ConventionRegistry,
    };

    #[derive(serde::Serialize, serde::Deserialize)]
    struct TestConvention;

    impl ZarrConventionImpl for TestConvention {
        const DEFINITION: ConventionDefinition = ConventionDefinition {
            uuid: uuid::uuid!("12345678-1234-5678-1234-567812345678"),
            schema_url: uri!("https://example.com/schemas/test_convention.json"),
            spec_url: uri!("https://example.com/specs/test_convention"),
            name: "test_convention",
            description: "A test convention.",
        };
    }

    #[test]
    fn test_register_and_get() {
        let registry = ConventionRegistry::default();
        registry.register::<TestConvention>().unwrap();

        let id = crate::ConventionId::Uuid(uuid::uuid!("12345678-1234-5678-1234-567812345678"));
        assert!(registry.contains(&id));

        let convention = registry.get(&id).expect("Convention not found");
        assert_eq!(convention.name, "test_convention");

        assert!(registry.register::<TestConvention>().is_err());
    }

    register_zarr_conventions!(TestConvention);

    #[test]
    fn test_registered_by_macro() {
        let id = crate::ConventionId::Uuid(uuid::uuid!("12345678-1234-5678-1234-567812345678"));
        assert!(crate::DEFAULT_ZARR_CONVENTION_REGISTRY.contains(&id));
    }
}
