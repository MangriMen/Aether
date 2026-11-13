//! The HTTP client's errors turned into a [`ProviderFailure`], keeping them as its source.

use crate::features::plugins::ProviderFailure;

impl From<reqwest::Error> for ProviderFailure {
    fn from(err: reqwest::Error) -> Self {
        Self::Request(Box::new(err))
    }
}

impl From<reqwest_middleware::Error> for ProviderFailure {
    fn from(value: reqwest_middleware::Error) -> Self {
        match value {
            reqwest_middleware::Error::Reqwest(err) => err.into(),
            // The middleware error is an opaque `anyhow::Error`; the typed causes it wraps are
            // in its chain.
            reqwest_middleware::Error::Middleware(err) => {
                Self::Request(Box::new(MiddlewareError(err)))
            }
        }
    }
}

/// An `anyhow::Error` as a source whose chain starts at the error it wraps, so that a typed
/// cause in it can be found by walking `source()`.
#[derive(Debug)]
struct MiddlewareError(anyhow::Error);

impl std::fmt::Display for MiddlewareError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for MiddlewareError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.0.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_keep_the_middleware_error_when_a_middleware_error_becomes_a_failure() {
        let failure = ProviderFailure::from(reqwest_middleware::Error::Middleware(
            anyhow::Error::new(std::io::Error::from(std::io::ErrorKind::TimedOut)),
        ));

        let ProviderFailure::Request(err) = failure else {
            panic!("expected a request failure, got {failure:?}");
        };
        assert!(
            err.source()
                .is_some_and(|source| source.downcast_ref::<std::io::Error>().is_some()),
            "{err:?}"
        );
    }

    #[test]
    fn should_keep_the_typed_error_when_a_reqwest_error_becomes_a_failure() {
        let err = reqwest::Client::new().get("not a url").build().unwrap_err();

        let failure = ProviderFailure::from(reqwest_middleware::Error::Reqwest(err));

        assert!(
            matches!(failure, ProviderFailure::Request(ref err)
                if err.downcast_ref::<reqwest::Error>().is_some()),
            "{failure:?}"
        );
    }
}
