/// [`extism::host_fn!`] for a host function that takes one structured argument, written as
/// `Wire<T>`. The argument is read as a [`HostArg`](super::super::mappers::HostArg): one that
/// does not decode is answered with `invalidArgument` and the body does not run, where extism's
/// own decoding would fail the host function and abort the plugin's call.
macro_rules! plugin_host_fn {
    ($pub:vis $name:ident($user_data:ident: $data:ty; $arg:ident: Wire<$arg_ty:ty>) -> $ret:ty $body:block) => {
        extism::host_fn!($pub $name($user_data: $data; $arg: $crate::features::plugins::infra::extism::mappers::HostArg<$arg_ty>) -> $ret {
            let $arg = match $arg.decode(stringify!($name)) {
                Ok(arg) => arg,
                Err(answer) => return Ok(answer),
            };
            $body
        });
    };
}

#[cfg(test)]
mod bad_argument_tests;
mod command_approval;
mod core;
mod download;
mod http;
mod instance;
mod java;
#[cfg(test)]
pub(crate) mod test_server;

pub use core::*;
pub use download::*;
pub use http::*;
pub use instance::*;
pub use java::*;
