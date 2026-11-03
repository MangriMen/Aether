use std::{fmt, ops::Deref};

use super::{InstanceError, InstanceField, InstanceValidationErrorReason};

/// A name an instance can be created with or renamed to.
///
/// The name is shown as it is, so only emptiness is refused: a name that cannot be a directory
/// is sanitized into an id instead of being rejected. The trimmed value is stored, so `" Pack "`
/// and `"Pack"` are the same name.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InstanceName(String);

impl InstanceName {
    pub fn parse(name: &str) -> Result<Self, InstanceError> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(InstanceError::ValidationError {
                field: InstanceField::Name,
                reason: InstanceValidationErrorReason::CannotBeEmpty,
            });
        }

        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }

    /// The name of a stored instance. A blank one (written before names were checked) falls back to
    /// the instance id, so the instance stays listed and can be renamed.
    pub fn from_stored_or_id(name: &str, id: &str) -> Self {
        Self::parse(name).unwrap_or_else(|_| {
            tracing::warn!("Instance {id} has a blank stored name; using its id instead");
            Self(id.trim().to_string())
        })
    }
}

impl TryFrom<String> for InstanceName {
    type Error = InstanceError;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        Self::parse(&name)
    }
}

impl fmt::Display for InstanceName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for InstanceName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for InstanceName {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_use_the_id_when_the_stored_name_is_blank() {
        assert_eq!(
            InstanceName::from_stored_or_id("  ", "pack").as_str(),
            "pack"
        );
        assert_eq!(
            InstanceName::from_stored_or_id(" My ", "pack").as_str(),
            "My"
        );
    }

    #[test]
    fn should_store_the_trimmed_name_when_parsed() {
        assert_eq!(
            InstanceName::parse("  My pack \t").unwrap().as_str(),
            "My pack"
        );
    }

    #[test]
    fn should_give_equal_names_when_they_differ_only_by_surrounding_spaces() {
        assert_eq!(
            InstanceName::parse(" Pack ").unwrap(),
            InstanceName::parse("Pack").unwrap()
        );
    }

    #[test]
    fn should_refuse_a_blank_name_when_parsed() {
        for blank in ["", "   ", "\t\n"] {
            let err = InstanceName::parse(blank).unwrap_err();
            assert!(matches!(
                err,
                InstanceError::ValidationError {
                    field: InstanceField::Name,
                    reason: InstanceValidationErrorReason::CannotBeEmpty,
                }
            ));
        }
    }
}
