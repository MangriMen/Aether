use aether_core_plugin_api::v0::OutputDto;

pub trait OutputDtoExt {
    fn from_output(output: &std::process::Output) -> Self;
}

impl OutputDtoExt for OutputDto {
    fn from_output(output: &std::process::Output) -> Self {
        Self {
            status: output.status.code().unwrap_or(0) as u32,
            stdout: output.stdout.clone(),
            stderr: output.stderr.clone(),
        }
    }
}
