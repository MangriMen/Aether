use crate::core::domain::HostResult;

#[allow(clippy::unnecessary_wraps)]
pub fn to_extism_res<T>(res: crate::Result<T>) -> Result<HostResult<T>, extism::Error> {
    Ok(HostResult::from(res))
}
