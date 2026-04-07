use crate::core::domain::HostResult;

pub fn to_extism_res<T>(res: crate::Result<T>) -> Result<HostResult<T>, extism::Error> {
    Ok(HostResult::from(res))
}
