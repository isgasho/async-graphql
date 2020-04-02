use crate::model::{__Schema, __Type};
use crate::{
    do_resolve, registry, Context, ContextSelectionSet, Error, ObjectType, OutputValueType,
    QueryError, Result, Type, Value,
};
use graphql_parser::query::Field;
use graphql_parser::Pos;
use std::borrow::Cow;
use std::collections::HashMap;

pub struct QueryRoot<T> {
    pub inner: T,
    pub disable_introspection: bool,
}

impl<T: Type> Type for QueryRoot<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        let schema_type = __Schema::create_type_info(registry);
        let root = T::create_type_info(registry);
        if let Some(registry::Type::Object { fields, .. }) =
            registry.types.get_mut(T::type_name().as_ref())
        {
            fields.insert(
                "__schema".to_string(),
                registry::Field {
                    name: "__schema".to_string(),
                    description: Some("Access the current type schema of this server."),
                    args: Default::default(),
                    ty: schema_type,
                    deprecation: None,
                    cache_control: Default::default(),
                },
            );

            fields.insert(
                "__type".to_string(),
                registry::Field {
                    name: "__type".to_string(),
                    description: Some("Request the type information of a single type."),
                    args: {
                        let mut args = HashMap::new();
                        args.insert(
                            "name",
                            registry::InputValue {
                                name: "name",
                                description: None,
                                ty: "String!".to_string(),
                                default_value: None,
                                validator: None,
                            },
                        );
                        args
                    },
                    ty: "__Type".to_string(),
                    deprecation: None,
                    cache_control: Default::default(),
                },
            );
        }
        root
    }
}

#[async_trait::async_trait]
impl<T: ObjectType + Send + Sync> ObjectType for QueryRoot<T> {
    async fn resolve_field(&self, ctx: &Context<'_>, field: &Field) -> Result<serde_json::Value> {
        if field.name.as_str() == "__schema" {
            if self.disable_introspection {
                return Err(Error::Query {
                    pos: field.position,
                    path: Some(ctx.path_node.as_ref().unwrap().to_json()),
                    err: QueryError::FieldNotFound {
                        field_name: field.name.clone(),
                        object: Self::type_name().to_string(),
                    },
                });
            }

            let ctx_obj = ctx.with_selection_set(&field.selection_set);
            return OutputValueType::resolve(
                &__Schema {
                    registry: &ctx.registry,
                },
                &ctx_obj,
                field.position,
            )
            .await;
        } else if field.name.as_str() == "__type" {
            let type_name: String = ctx.param_value("name", field.position, || Value::Null)?;
            let ctx_obj = ctx.with_selection_set(&field.selection_set);
            return OutputValueType::resolve(
                &ctx.registry
                    .types
                    .get(&type_name)
                    .map(|ty| __Type::new_simple(ctx.registry, ty)),
                &ctx_obj,
                field.position,
            )
            .await;
        }

        self.inner.resolve_field(ctx, field).await
    }
}

#[async_trait::async_trait]
impl<T: ObjectType + Send + Sync> OutputValueType for QueryRoot<T> {
    async fn resolve(
        value: &Self,
        ctx: &ContextSelectionSet<'_>,
        _pos: Pos,
    ) -> Result<serde_json::Value> {
        do_resolve(ctx, value).await
    }
}
