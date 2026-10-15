use aether_core_plugin_api::v0::OutputDto;

pub(crate) trait OutputDtoExt {
    fn from_output(
        output: &std::process::Output,
        stdout_truncated: bool,
        stderr_truncated: bool,
    ) -> Self;
}

impl OutputDtoExt for OutputDto {
    fn from_output(
        output: &std::process::Output,
        stdout_truncated: bool,
        stderr_truncated: bool,
    ) -> Self {
        Self {
            status: u32::try_from(output.status.code().unwrap_or(0)).unwrap_or(0),
            stdout: output.stdout.clone(),
            stderr: output.stderr.clone(),
            stdout_truncated,
            stderr_truncated,
        }
    }
}

/// `OutputDto` crosses the plugin boundary as `Msgpack`, which `extism-convert` encodes as a
/// fixed-length array. These pin down what that means for the truncation flags.
#[cfg(test)]
mod tests {
    use extism_convert::{FromBytes, Msgpack, ToBytes};
    use serde::{Deserialize, Serialize};

    use super::*;

    /// `OutputDto` without the truncation flags.
    #[derive(Serialize, Deserialize, Debug)]
    struct OutputDtoWithoutFlags {
        status: u32,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    }

    fn encode<T: Serialize>(value: T) -> Vec<u8> {
        Msgpack(value).to_bytes().expect("DTO should encode")
    }

    /// A plugin built against the new DTO still reads a response from a host without the
    /// flags: the missing tail falls back to `false`.
    #[test]
    fn should_decode_a_response_as_not_truncated_when_it_has_no_flags() {
        let bytes = encode(OutputDtoWithoutFlags {
            status: 0,
            stdout: b"out".to_vec(),
            stderr: b"err".to_vec(),
        });

        let Msgpack(output) = Msgpack::<OutputDto>::from_bytes(&bytes).expect("should decode");

        assert_eq!(output.stdout, b"out");
        assert!(!output.stdout_truncated && !output.stderr_truncated);
    }

    #[test]
    fn should_carry_each_flag_across_the_boundary_when_encoded_and_decoded() {
        let bytes = encode(OutputDto {
            status: 0,
            stdout: vec![],
            stderr: vec![],
            stdout_truncated: false,
            stderr_truncated: true,
        });

        let Msgpack(output) = Msgpack::<OutputDto>::from_bytes(&bytes).expect("should decode");

        assert!(!output.stdout_truncated);
        assert!(output.stderr_truncated);
    }

    /// The other direction breaks: a plugin built without the flags cannot decode a response
    /// that has them, so it has to be rebuilt against the current `aether-core-plugin-api`.
    #[test]
    fn should_not_be_readable_by_a_plugin_built_before_the_flags_when_the_flags_are_present() {
        let bytes = encode(OutputDto {
            status: 0,
            stdout: vec![],
            stderr: vec![],
            stdout_truncated: false,
            stderr_truncated: false,
        });

        assert!(Msgpack::<OutputDtoWithoutFlags>::from_bytes(&bytes).is_err());
    }
}
