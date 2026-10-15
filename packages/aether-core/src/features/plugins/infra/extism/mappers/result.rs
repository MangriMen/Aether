use aether_core_plugin_api::v0::{HostError, HostResult};
use extism_convert::ToBytes;
use serde::Serialize;

/// A host function's result as the plugin receives it.
///
/// Msgpack through `extism_convert::Msgpack` writes structs as arrays, and serde cannot read an
/// adjacently tagged enum back from an array when the variant has no content: every error
/// variant without fields (`HostError::AuthError(TokenExpired)`) would fail to decode in the
/// plugin. Errors are therefore written as maps, which every serde reader accepts; the success
/// path keeps the compact form.
#[derive(Debug)]
pub(crate) struct MsgpackResult<T>(pub HostResult<T>);

impl<T: Serialize> ToBytes<'_> for MsgpackResult<T> {
    type Bytes = Vec<u8>;

    fn to_bytes(&self) -> Result<Self::Bytes, extism_convert::Error> {
        let bytes = match &self.0 {
            HostResult::Ok(_) => rmp_serde::to_vec(&self.0)?,
            HostResult::Err(_) => rmp_serde::to_vec_named(&self.0)?,
        };
        Ok(bytes)
    }
}

#[allow(clippy::unnecessary_wraps)]
pub(crate) fn to_extism_res<T>(res: crate::Result<T>) -> Result<MsgpackResult<T>, extism::Error> {
    let host_res = match res {
        Ok(v) => HostResult::Ok(v),
        Err(e) => HostResult::Err(HostError::from(e.raw.as_ref())),
    };

    Ok(MsgpackResult(host_res))
}

#[cfg(test)]
mod tests {
    use aether_core_plugin_api::v0::{
        AuthErrorDto, EventErrorDto, FileWatcherErrorDto, HostError, HostResult, InstanceErrorDto,
        JavaErrorDto, MinecraftErrorDto, PluginCallErrorDto, PluginErrorDto, ProcessErrorDto,
        RequestErrorDto, SettingsErrorDto,
    };
    use extism_convert::{FromBytesOwned, Msgpack, ToBytes};

    use super::to_extism_res;
    use crate::{
        ErrorKind,
        features::{
            auth::{AuthApplicationError, AuthDomainError},
            events::EventError,
            file_watcher::FileWatcherError,
            instance::InstanceError,
            java::{JavaApplicationError, JavaDomainError},
            minecraft::{MinecraftApplicationError, MinecraftDomainError},
            plugins::{PluginCallError, PluginError},
            process::ProcessError,
            settings::SettingsError,
        },
        shared::request_client::RequestError,
    };

    /// What the plugin sees: the error after the msgpack the host function returns.
    fn through_msgpack(kind: ErrorKind) -> HostError {
        let res: crate::Result<()> = Err(kind.as_error());
        let bytes = to_extism_res(res)
            .expect("mapping never fails")
            .to_bytes()
            .expect("encodes");
        let Msgpack(decoded) =
            <Msgpack<HostResult<()>>>::from_bytes_owned(bytes.as_ref()).expect("decodes");
        match decoded {
            HostResult::Err(error) => error,
            HostResult::Ok(()) => panic!("an error became Ok"),
        }
    }

    /// The bytes of a file or a response cross the plugin boundary as a msgpack `bin`: as an
    /// array of numbers the plugin would spend fuel on every byte (a 1.5 MiB jar used up the
    /// whole budget of an `import` call).
    #[test]
    fn should_hand_bytes_over_as_a_msgpack_bin_when_encoding_a_result() {
        let body = vec![7_u8; 4096];
        let response = aether_core_plugin_api::v0::HttpResponseDto {
            status: 200,
            headers: std::collections::HashMap::new(),
            body: body.clone(),
        };

        let bytes = Msgpack(&response).to_bytes().expect("encodes");

        assert!(
            bytes.len() < body.len() + 64,
            "{} bytes for a {}-byte body: the body is not a bin",
            bytes.len(),
            body.len()
        );
        let Msgpack(decoded) =
            <Msgpack<aether_core_plugin_api::v0::HttpResponseDto>>::from_bytes_owned(
                bytes.as_ref(),
            )
            .expect("decodes");
        assert_eq!(decoded.body, body);
    }

    #[test]
    fn should_still_read_bytes_when_they_were_written_as_an_array_of_numbers() {
        let old = rmp_serde::to_vec(&(
            200_u16,
            std::collections::HashMap::<String, String>::new(),
            vec![1_u8, 2, 3],
        ))
        .expect("encodes");

        let Msgpack(decoded) =
            <Msgpack<aether_core_plugin_api::v0::HttpResponseDto>>::from_bytes_owned(&old)
                .expect("decodes");

        assert_eq!(decoded.body, [1, 2, 3]);
    }

    #[test]
    fn should_hand_a_core_error_over_without_the_display_prefix_when_mapping_it() {
        let host = through_msgpack(ErrorKind::CoreError("host is not allowed".to_owned()));

        assert!(
            matches!(&host, HostError::CoreError(message) if message == "host is not allowed"),
            "{host:?}"
        );
    }

    #[test]
    fn should_keep_the_variant_when_mapping_an_auth_error() {
        let host = through_msgpack(
            AuthApplicationError::Domain(AuthDomainError::InvalidUsernameLength {
                min: 3,
                max: 16,
            })
            .into(),
        );

        assert!(
            matches!(
                host,
                HostError::AuthError(AuthErrorDto::InvalidUsernameLength { min: 3, max: 16 })
            ),
            "{host:?}"
        );
    }

    #[test]
    fn should_keep_the_variant_when_mapping_an_event_error() {
        let host = through_msgpack(EventError::NotInitialized.into());

        assert!(
            matches!(host, HostError::EventError(EventErrorDto::NotInitialized)),
            "{host:?}"
        );
    }

    #[test]
    fn should_keep_the_variant_when_mapping_a_file_watcher_error() {
        let host = through_msgpack(FileWatcherError::WatchNotFound.into());

        assert!(
            matches!(
                host,
                HostError::FileWatcherError(FileWatcherErrorDto::WatchNotFound)
            ),
            "{host:?}"
        );
    }

    #[test]
    fn should_keep_the_variant_and_fields_when_mapping_an_instance_error() {
        let host = through_msgpack(
            InstanceError::PathEscapesInstance {
                path: "../x".to_owned(),
                instance_id: "inst".to_owned(),
            }
            .into(),
        );

        assert!(
            matches!(
                &host,
                HostError::InstanceError(InstanceErrorDto::PathEscapesInstance { path, instance_id })
                    if path == "../x" && instance_id == "inst"
            ),
            "{host:?}"
        );
    }

    #[test]
    fn should_keep_a_u64_field_when_mapping_an_instance_error() {
        let host = through_msgpack(
            InstanceError::FileTooLarge {
                path: "a.jar".to_owned(),
                size: u64::MAX,
                limit: 7,
            }
            .into(),
        );

        assert!(
            matches!(
                host,
                HostError::InstanceError(InstanceErrorDto::FileTooLarge {
                    size: u64::MAX,
                    limit: 7,
                    ..
                })
            ),
            "{host:?}"
        );
    }

    #[test]
    fn should_hand_an_error_over_as_its_own_feature_when_an_instance_error_wraps_it() {
        let host = through_msgpack(
            InstanceError::MinecraftError(MinecraftApplicationError::Domain(
                MinecraftDomainError::VersionNotFound {
                    version: "1.20".to_owned(),
                },
            ))
            .into(),
        );

        assert!(
            matches!(
                &host,
                HostError::MinecraftError(MinecraftErrorDto::VersionNotFound { version })
                    if version == "1.20"
            ),
            "{host:?}"
        );
    }

    #[test]
    fn should_hand_a_download_failure_over_as_a_request_error_when_java_reports_it() {
        let host = through_msgpack(
            JavaApplicationError::DownloadFailed(RequestError::Status {
                status: 404,
                url: "https://example.com/jre".to_owned(),
            })
            .into(),
        );

        assert!(
            matches!(
                &host,
                HostError::RequestError(RequestErrorDto::Status { status: 404, url })
                    if url == "https://example.com/jre"
            ),
            "{host:?}"
        );
    }

    #[test]
    fn should_keep_the_variant_when_mapping_a_java_error() {
        let host = through_msgpack(
            JavaApplicationError::Domain(JavaDomainError::NotFound { version: 17 }).into(),
        );

        assert!(
            matches!(
                host,
                HostError::JavaError(JavaErrorDto::NotFound { version: 17 })
            ),
            "{host:?}"
        );
    }

    #[test]
    fn should_keep_the_variant_and_call_error_when_mapping_a_plugin_error() {
        let host = through_msgpack(
            PluginError::FunctionCallFailed {
                function_name: "install".to_owned(),
                plugin_id: "packwiz".to_owned(),
                error: PluginCallError::Timeout,
            }
            .into(),
        );

        assert!(
            matches!(
                &host,
                HostError::PluginError(PluginErrorDto::FunctionCallFailed {
                    error: PluginCallErrorDto::Timeout,
                    plugin_id,
                    ..
                }) if plugin_id == "packwiz"
            ),
            "{host:?}"
        );
    }

    #[test]
    fn should_hand_a_settings_error_over_as_a_settings_error_when_a_plugin_error_wraps_it() {
        let host = through_msgpack(PluginError::Settings(SettingsError::NotFound).into());

        assert!(
            matches!(host, HostError::SettingsError(SettingsErrorDto::NotFound)),
            "{host:?}"
        );
    }

    #[test]
    fn should_keep_the_variant_when_mapping_a_process_error() {
        let host = through_msgpack(
            ProcessError::KillError {
                id: "p1".to_owned(),
            }
            .into(),
        );

        assert!(
            matches!(
                &host,
                HostError::ProcessError(ProcessErrorDto::KillError { id }) if id == "p1"
            ),
            "{host:?}"
        );
    }

    #[test]
    fn should_keep_the_variant_when_mapping_a_settings_error() {
        let host = through_msgpack(SettingsError::Storage("disk".to_owned()).into());

        assert!(
            matches!(&host, HostError::SettingsError(SettingsErrorDto::Storage(m)) if m == "disk"),
            "{host:?}"
        );
    }

    #[test]
    fn should_keep_the_variant_and_u64_when_mapping_a_request_error() {
        let host = through_msgpack(RequestError::TooLarge { limit: u64::MAX }.into());

        assert!(
            matches!(
                host,
                HostError::RequestError(RequestErrorDto::TooLarge { limit: u64::MAX })
            ),
            "{host:?}"
        );
    }
}
