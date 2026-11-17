//! The host functions, one safe function each: the argument is encoded with [`Wire`], the
//! answer decoded, and a host's refusal comes back as [`Error::Host`] for `?` to carry on.
//!
//! ```ignore
//! use aether_core_plugin_pdk::{self as pdk, ResultExt, api::v0::HttpRequestDto};
//!
//! fn fetch(url: &str) -> pdk::Result<Vec<u8>> {
//!     let response = pdk::host::http_get(HttpRequestDto {
//!         url: url.to_owned(),
//!         headers: Default::default(),
//!     })
//!     .context("fetching the pack index")?;
//!     Ok(response.body)
//! }
//! ```
//!
//! A function here imports its host function into the plugin's wasm only when the plugin calls
//! it (the linker drops the rest), and `api.hostFunctions` of the manifest must list every
//! import, so list exactly the ones the plugin calls.
//!
//! The host functions exist only inside the launcher: built for anything but `wasm32` (a plugin's
//! native unit tests), every function here fails with an [`Error::Message`] that names it,
//! without calling anything.

use aether_core_plugin_api::v0::{
    CachedFileDto, ChangeContentsParamsDto, CommandDto, DownloadToCacheParamsDto,
    HttpPostJsonRequestDto, HttpRequestDto, HttpResponseDto, InstanceCreatedDto, InstanceDirDto,
    InstanceFileBytesDto, InstanceIdParamsDto, JavaDto, JavaVersionParamsDto, ListContentResultDto,
    LogLevelDto, LogParamsDto, NewInstanceDto, OutputDto, ReadInstanceFileParamsDto,
    WriteInstanceFileParamsDto,
};

use crate::{Error, HostReply, Result, Wire};

/// The raw imports, as `#[host_fn]` declares them, and the same functions failing outright
/// outside wasm, from one list so that the two cannot drift apart.
macro_rules! imports {
    ($($name:ident($param:ty) -> $reply:ty;)*) => {
        #[cfg(target_arch = "wasm32")]
        #[extism_pdk::host_fn]
        extern "ExtismHost" {
            $(pub(crate) fn $name(params: Wire<$param>) -> HostReply<$reply>;)*
        }

        $(
            #[cfg(not(target_arch = "wasm32"))]
            pub(crate) unsafe fn $name(
                _params: Wire<$param>,
            ) -> std::result::Result<HostReply<$reply>, extism_pdk::Error> {
                Err(anyhow::anyhow!(
                    "the host function `{}` exists only in a wasm plugin run by the launcher",
                    stringify!($name)
                ))
            }
        )*
    };
}

mod raw {
    use super::{
        CachedFileDto, ChangeContentsParamsDto, CommandDto, DownloadToCacheParamsDto, HostReply,
        HttpPostJsonRequestDto, HttpRequestDto, HttpResponseDto, InstanceCreatedDto,
        InstanceDirDto, InstanceFileBytesDto, InstanceIdParamsDto, JavaDto, JavaVersionParamsDto,
        ListContentResultDto, LogParamsDto, NewInstanceDto, OutputDto, ReadInstanceFileParamsDto,
        Wire, WriteInstanceFileParamsDto,
    };

    imports! {
        log(LogParamsDto) -> ();
        run_command(CommandDto) -> OutputDto;
        get_java(JavaVersionParamsDto) -> JavaDto;
        install_java(JavaVersionParamsDto) -> JavaDto;
        instance_get_dir(InstanceIdParamsDto) -> InstanceDirDto;
        instance_plugin_get_dir(InstanceIdParamsDto) -> InstanceDirDto;
        instance_create(NewInstanceDto) -> InstanceCreatedDto;
        list_content(InstanceIdParamsDto) -> ListContentResultDto;
        enable_contents(ChangeContentsParamsDto) -> ();
        disable_contents(ChangeContentsParamsDto) -> ();
        write_instance_file(WriteInstanceFileParamsDto) -> ();
        read_instance_file(ReadInstanceFileParamsDto) -> InstanceFileBytesDto;
        http_get(HttpRequestDto) -> HttpResponseDto;
        http_post_json(HttpPostJsonRequestDto) -> HttpResponseDto;
        download_to_cache(DownloadToCacheParamsDto) -> CachedFileDto;
    }
}

/// What a raw call ended with, as the plugin reads it: the value, the host's error unchanged,
/// or a call that did not get through (the argument could not be encoded, the answer could not
/// be read), which names the host function.
fn answer<T>(name: &str, reply: std::result::Result<HostReply<T>, extism_pdk::Error>) -> Result<T> {
    reply
        .map_err(|error| Error::from(error).context(format!("calling the host function `{name}`")))?
        .into_result()
}

/// Calls one host function. The raw import is `unsafe` only because it is `extern`: the
/// argument is an owned value the macro copies into Extism memory, and the answer is read back
/// from the memory the host returned, so nothing is left for the caller to uphold.
macro_rules! call {
    ($name:ident($params:expr)) => {
        // SAFETY: see `call!`.
        answer(stringify!($name), unsafe { raw::$name(Wire($params)) })
    };
}

/// `log`: writes `message` to the launcher's log at `level`, under the plugin's id.
pub fn log(level: LogLevelDto, message: impl Into<String>) -> Result<()> {
    call!(log(LogParamsDto {
        level,
        message: message.into(),
    }))
}

/// `run_command`: runs a Java launcher once the user allows it (see "`run_command` input" and
/// "`run_command` needs the user's consent" in the API README). A non-zero exit is an answer
/// (`OutputDto::exit_code`), not a failure; a refusal is `commandNotApproved`.
pub fn run_command(command: CommandDto) -> Result<OutputDto> {
    call!(run_command(command))
}

/// `get_java`: the Java runtime of that major `version` (8, 17, 21, …) registered with the
/// launcher.
pub fn get_java(version: u32) -> Result<JavaDto> {
    call!(get_java(JavaVersionParamsDto { version }))
}

/// `install_java`: installs the Java runtime of that major `version` and answers it.
pub fn install_java(version: u32) -> Result<JavaDto> {
    call!(install_java(JavaVersionParamsDto { version }))
}

/// `instance_get_dir`: the sandbox path of the instance, `/instances/<id>`.
pub fn instance_get_dir(instance_id: impl Into<String>) -> Result<InstanceDirDto> {
    call!(instance_get_dir(InstanceIdParamsDto {
        instance_id: instance_id.into(),
    }))
}

/// `instance_plugin_get_dir`: the sandbox path of the plugin's own folder in the instance,
/// `/instances/<id>/.metadata/plugins/<plugin id>`.
pub fn instance_plugin_get_dir(instance_id: impl Into<String>) -> Result<InstanceDirDto> {
    call!(instance_plugin_get_dir(InstanceIdParamsDto {
        instance_id: instance_id.into(),
    }))
}

/// `instance_create`: creates an instance and answers its id.
pub fn instance_create(instance: NewInstanceDto) -> Result<InstanceCreatedDto> {
    call!(instance_create(instance))
}

/// `list_content`: the content files of the instance.
pub fn list_content(instance_id: impl Into<String>) -> Result<ListContentResultDto> {
    call!(list_content(InstanceIdParamsDto {
        instance_id: instance_id.into(),
    }))
}

/// `enable_contents`: enables the content files named by `paths`, as `list_content` names them.
pub fn enable_contents(params: ChangeContentsParamsDto) -> Result<()> {
    call!(enable_contents(params))
}

/// `disable_contents`: disables the content files named by `paths`, as `list_content` names them.
pub fn disable_contents(params: ChangeContentsParamsDto) -> Result<()> {
    call!(disable_contents(params))
}

/// `write_instance_file`: replaces a file of the instance, by a path relative to the instance
/// (see "Instance files" in the API README).
pub fn write_instance_file(params: WriteInstanceFileParamsDto) -> Result<()> {
    call!(write_instance_file(params))
}

/// `read_instance_file`: the bytes of a file of the instance, by a path relative to the
/// instance.
pub fn read_instance_file(params: ReadInstanceFileParamsDto) -> Result<Vec<u8>> {
    call!(read_instance_file(params)).map(|InstanceFileBytesDto(bytes)| bytes)
}

/// `http_get`: a GET the host makes under the plugin's `allowedHosts` (see "Networking from a
/// plugin" in the API README). Any non-3xx status is an answer, not a failure.
pub fn http_get(request: HttpRequestDto) -> Result<HttpResponseDto> {
    call!(http_get(request))
}

/// `http_post_json`: a JSON POST under the same policy as [`http_get`], following no redirect.
pub fn http_post_json(request: HttpPostJsonRequestDto) -> Result<HttpResponseDto> {
    call!(http_post_json(request))
}

/// `download_to_cache`: downloads a file into `/cache`, checked against its hash; the bytes
/// never pass through the plugin's memory.
pub fn download_to_cache(params: DownloadToCacheParamsDto) -> Result<CachedFileDto> {
    call!(download_to_cache(params))
}

#[cfg(test)]
mod tests {
    use aether_core_plugin_api::v0::{HostError, HostResult, wire};
    use extism_convert::FromBytesOwned;

    use super::*;

    fn reply<T: serde::Serialize + serde::de::DeserializeOwned>(
        result: &HostResult<T>,
    ) -> HostReply<T> {
        HostReply::from_bytes_owned(&wire::encode(result).unwrap()).unwrap()
    }

    #[test]
    fn an_ok_answer_is_the_value() {
        let dir = InstanceDirDto {
            path: "/instances/a".to_owned(),
        };

        let value = answer("instance_get_dir", Ok(reply(&HostResult::Ok(dir.clone()))));

        assert_eq!(value.unwrap(), dir);
    }

    #[test]
    fn an_err_answer_is_the_host_error_without_context() {
        let error = HostError::NotFound {
            message: "no instance `a`".to_owned(),
        };

        let value = answer(
            "instance_get_dir",
            Ok(reply(&HostResult::<InstanceDirDto>::Err(error.clone()))),
        );

        assert!(
            matches!(&value, Err(Error::Host { error: got, context: None }) if *got == error),
            "{value:?}"
        );
    }

    #[test]
    fn a_call_that_did_not_get_through_names_the_host_function() {
        let value = answer::<()>("log", Err(anyhow::anyhow!("out of memory")));

        assert!(
            matches!(&value, Err(Error::Message(message))
                if message == "calling the host function `log`: out of memory"),
            "{value:?}"
        );
    }

    #[test]
    fn outside_wasm_a_host_function_fails_without_calling_anything() {
        let value = get_java(21);

        assert!(
            matches!(&value, Err(Error::Message(message))
                if message.contains("`get_java` exists only in a wasm plugin")),
            "{value:?}"
        );
    }
}
