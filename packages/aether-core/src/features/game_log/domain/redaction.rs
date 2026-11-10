/// Secrets shorter than this are not redacted: the offline account's token is the literal
/// `null`, and replacing that word would damage every line.
const MIN_SECRET_LEN: usize = 20;

const MASK: &str = "***";

#[derive(Debug, Clone, Default)]
pub struct Redactor {
    secrets: Vec<String>,
}

impl Redactor {
    pub fn new(secrets: impl IntoIterator<Item = String>) -> Self {
        Self {
            secrets: secrets
                .into_iter()
                .filter(|secret| secret.len() >= MIN_SECRET_LEN)
                .collect(),
        }
    }

    pub fn apply(&self, line: &str) -> String {
        self.secrets
            .iter()
            .fold(line.to_string(), |acc, secret| acc.replace(secret, MASK))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_mask_the_secret_when_a_line_contains_it() {
        let token = "a".repeat(30);
        let redactor = Redactor::new([token.clone()]);

        assert_eq!(
            redactor.apply(&format!("Session ID is token:{token}:id")),
            "Session ID is token:***:id"
        );
    }

    #[test]
    fn should_leave_lines_alone_when_the_secret_is_too_short() {
        let redactor = Redactor::new(["null".to_string()]);

        assert_eq!(redactor.apply("null pointer"), "null pointer");
    }
}
