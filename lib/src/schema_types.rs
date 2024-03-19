use schemars::{
    gen::SchemaGenerator,
    schema::{InstanceType, Schema, SchemaObject, StringValidation},
    JsonSchema,
};
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    borrow::Cow,
    fmt::Display,
    ops::{Deref, DerefMut},
};

#[derive(Debug, PartialEq, Clone)]
pub struct VersionReq(pub semver::VersionReq);

impl Serialize for VersionReq {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl VersionReq {
    pub const STAR: Self = VersionReq(semver::VersionReq::STAR);

    pub fn parse(text: &str) -> Result<Self, semver::Error> {
        Ok(VersionReq(semver::VersionReq::parse(text)?))
    }

    pub fn matches(&self, version: &semver::Version) -> bool {
        semver::VersionReq::matches(self, version)
    }
}

impl Display for VersionReq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Deref for VersionReq {
    type Target = semver::VersionReq;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VersionReq {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'de> Deserialize<'de> for VersionReq {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct VersionReqVisitor;

        impl<'de> serde::de::Visitor<'de> for VersionReqVisitor {
            type Value = VersionReq;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a semver version as a string")
            }

            fn visit_str<E>(self, value: &str) -> Result<VersionReq, E>
            where
                E: serde::de::Error,
            {
                semver::VersionReq::parse(value)
                    .map(VersionReq)
                    .map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(VersionReqVisitor)
    }
}

impl JsonSchema for VersionReq {
    fn schema_name() -> String {
        "VersionReq".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("version_req")
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            string: Some(Box::new(StringValidation::default())),
            ..Default::default()
        }
        .into()
    }
}
