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

/// `OutputDto` crosses the plugin boundary as a msgpack map, so either side can add a field.
#[cfg(test)]
mod tests {
    use aether_core_plugin_api::v0::wire;
    use serde::{Deserialize, Serialize};

    use super::*;

    /// `OutputDto` without the truncation flags.
    #[derive(Serialize, Deserialize, Debug)]
    struct OutputDtoWithoutFlags {
        status: u32,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    }

    /// A plugin built against the new DTO still reads a response from a host without the
    /// flags: the missing keys fall back to `false`.
    #[test]
    fn should_decode_a_response_as_not_truncated_when_it_has_no_flags() {
        let bytes = wire::encode(&OutputDtoWithoutFlags {
            status: 0,
            stdout: b"out".to_vec(),
            stderr: b"err".to_vec(),
        })
        .expect("DTO should encode");

        let output: OutputDto = wire::decode(&bytes).expect("should decode");

        assert_eq!(output.stdout, b"out");
        assert!(!output.stdout_truncated && !output.stderr_truncated);
    }

    #[test]
    fn should_carry_each_flag_across_the_boundary_when_encoded_and_decoded() {
        let bytes = wire::encode(&OutputDto {
            status: 0,
            stdout: vec![],
            stderr: vec![],
            stdout_truncated: false,
            stderr_truncated: true,
        })
        .expect("DTO should encode");

        let output: OutputDto = wire::decode(&bytes).expect("should decode");

        assert!(!output.stdout_truncated);
        assert!(output.stderr_truncated);
    }

    /// The other direction works too: a plugin built without the flags ignores the keys it does
    /// not know.
    #[test]
    fn should_be_readable_by_a_plugin_built_before_the_flags_when_the_flags_are_present() {
        let bytes = wire::encode(&OutputDto {
            status: 0,
            stdout: vec![],
            stderr: vec![],
            stdout_truncated: false,
            stderr_truncated: false,
        })
        .expect("DTO should encode");

        assert!(wire::decode::<OutputDtoWithoutFlags>(&bytes).is_ok());
    }
}
