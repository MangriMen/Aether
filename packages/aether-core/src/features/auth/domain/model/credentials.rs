use std::fmt::Display;

use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::features::auth::domain::AuthDomainError;

use super::Username;

#[derive(Clone, PartialEq, Eq)]
pub struct Credential {
    id: Uuid,
    username: Username,
    account_type: AccountType,
    is_active: bool,
    access_token: String,
    refresh_token: String,
    expires: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountType {
    Offline,
    Microsoft,
}

impl Credential {
    pub fn new(
        id: Uuid,
        username: Username,
        account_type: AccountType,
        is_active: bool,
        access_token: String,
        refresh_token: String,
        expires: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            username,
            account_type,
            is_active,
            access_token,
            refresh_token,
            expires,
        }
    }

    pub fn new_offline(id: Uuid, username: Username) -> Self {
        Self {
            id,
            username,
            is_active: false,
            account_type: AccountType::Offline,
            access_token: "null".to_string(),
            refresh_token: "null".to_string(),
            expires: Utc::now() + Duration::days(365 * 99),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn username(&self) -> &Username {
        &self.username
    }
    pub fn account_type(&self) -> AccountType {
        self.account_type
    }
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    pub fn access_token(&self) -> &str {
        &self.access_token
    }
    pub fn refresh_token(&self) -> &str {
        &self.refresh_token
    }
    pub fn expires(&self) -> DateTime<Utc> {
        self.expires
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires
    }

    pub fn activate(&mut self) -> Result<(), AuthDomainError> {
        match self.account_type {
            AccountType::Microsoft => {
                if self.is_expired() {
                    return Err(AuthDomainError::TokenExpired);
                }
            }
            AccountType::Offline => (),
        }

        self.is_active = true;

        Ok(())
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    pub fn update_tokens(&mut self, access: String, refresh: String, expires_in: i64) {
        self.access_token = access;
        self.refresh_token = refresh;
        self.expires = Utc::now() + chrono::Duration::seconds(expires_in);
    }
}

/// The tokens are left out: a `{:?}` in a log line, a panic message or a bug report must not leak
/// the account.
impl std::fmt::Debug for Credential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credential")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("account_type", &self.account_type)
            .field("is_active", &self.is_active)
            .field("access_token", &"<redacted>")
            .field("refresh_token", &"<redacted>")
            .field("expires", &self.expires)
            .finish()
    }
}

impl AccountType {
    pub fn from_string(account_type: &str) -> Result<AccountType, AuthDomainError> {
        match account_type {
            "offline" => Ok(AccountType::Offline),
            "microsoft" => Ok(AccountType::Microsoft),
            _ => Err(AuthDomainError::InvalidAccountType),
        }
    }
}

impl Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            AccountType::Offline => "offline".to_string(),
            AccountType::Microsoft => "microsoft".to_string(),
        };

        write!(f, "{string}")
    }
}

impl From<AccountType> for String {
    fn from(value: AccountType) -> Self {
        value.to_string()
    }
}

impl TryFrom<String> for AccountType {
    type Error = AuthDomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_string(&value)
    }
}

#[cfg(test)]
mod account_type_tests {
    use super::*;

    #[test]
    fn should_display_account_type_name_when_formatted() {
        assert_eq!(AccountType::Offline.to_string(), "offline");
        assert_eq!(AccountType::Microsoft.to_string(), "microsoft");
    }

    #[test]
    fn should_parse_account_type_when_string_is_valid() {
        assert_eq!(
            AccountType::from_string("offline").unwrap(),
            AccountType::Offline
        );
        assert_eq!(
            AccountType::from_string("microsoft").unwrap(),
            AccountType::Microsoft
        );
    }

    #[test]
    fn should_fail_to_parse_account_type_when_string_is_invalid() {
        let err = AccountType::from_string("unknown").unwrap_err();
        assert!(matches!(err, AuthDomainError::InvalidAccountType));
    }

    #[test]
    fn should_convert_string_to_account_type_when_it_is_a_known_name() {
        let result: Result<AccountType, _> = "microsoft".to_string().try_into();
        assert_eq!(result.unwrap(), AccountType::Microsoft);
    }

    #[test]
    fn should_convert_account_type_to_string_when_it_is_offline() {
        let s: String = AccountType::Offline.into();
        assert_eq!(s, "offline");
    }
}

#[cfg(test)]
mod credential_tests {
    use super::*;

    #[test]
    fn should_not_show_tokens_when_formatted_for_debug() {
        let cred = Credential::new(
            Uuid::new_v4(),
            Username::parse("MSPlayer").unwrap(),
            AccountType::Microsoft,
            false,
            "secret-access".into(),
            "secret-refresh".into(),
            Utc::now(),
        );

        let debug = format!("{cred:?}");

        assert!(!debug.contains("secret-access"));
        assert!(!debug.contains("secret-refresh"));
        assert!(debug.contains("MSPlayer"));
    }

    #[test]
    fn should_activate_offline_credential_when_it_is_inactive() {
        let id = Uuid::new_v4();
        let mut cred = Credential::new_offline(id, Username::parse("Player").unwrap());

        assert!(!cred.is_active());
        cred.activate().unwrap();
        assert!(cred.is_active());
    }

    #[test]
    fn should_activate_microsoft_credential_when_not_expired() {
        let id = Uuid::new_v4();
        let expires = Utc::now() + Duration::hours(1);
        let mut cred = Credential::new(
            id,
            Username::parse("MSPlayer").unwrap(),
            AccountType::Microsoft,
            false,
            "access".into(),
            "refresh".into(),
            expires,
        );

        assert!(!cred.is_active());
        cred.activate().unwrap();
        assert!(cred.is_active());
    }

    #[test]
    fn should_set_the_offline_defaults_when_created_offline() {
        let id = Uuid::new_v4();

        let cred = Credential::new_offline(id, Username::parse("Player").unwrap());

        assert_eq!(cred.id(), id);
        assert_eq!(cred.account_type(), AccountType::Offline);
        assert!(!cred.is_active());
        assert!(!cred.is_expired());
    }

    #[test]
    fn should_deactivate_the_credential_when_it_is_active() {
        let mut cred = Credential::new_offline(Uuid::new_v4(), Username::parse("Player").unwrap());
        cred.activate().unwrap();

        cred.deactivate();

        assert!(!cred.is_active());
    }

    #[test]
    fn should_replace_the_tokens_and_extend_the_expiry_when_they_are_updated() {
        let mut cred = Credential::new(
            Uuid::new_v4(),
            Username::parse("MSPlayer").unwrap(),
            AccountType::Microsoft,
            true,
            "old-access".into(),
            "old-refresh".into(),
            Utc::now() - Duration::hours(1),
        );
        assert!(cred.is_expired());

        cred.update_tokens("new-access".into(), "new-refresh".into(), 3600);

        assert_eq!(cred.access_token(), "new-access");
        assert_eq!(cred.refresh_token(), "new-refresh");
        assert!(!cred.is_expired());
        assert!(cred.expires() > Utc::now() + Duration::minutes(59));
        assert!(cred.is_active(), "the activity is untouched");
    }

    #[test]
    fn should_fail_activation_when_microsoft_token_expired() {
        let id = Uuid::new_v4();
        let expires = Utc::now() - Duration::hours(1);
        let mut cred = Credential::new(
            id,
            Username::parse("MSPlayer").unwrap(),
            AccountType::Microsoft,
            false,
            "access".into(),
            "refresh".into(),
            expires,
        );

        assert!(cred.is_expired());
        let err = cred.activate().unwrap_err();
        assert!(matches!(err, AuthDomainError::TokenExpired));
        assert!(!cred.is_active());
    }
}
