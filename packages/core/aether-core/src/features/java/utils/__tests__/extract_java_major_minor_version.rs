use crate::features::java::{utils::extract_java_major_minor_version, JavaDomainError};

#[test]
fn check_java_8() -> Result<(), JavaDomainError> {
    let (major, minor) = extract_java_major_minor_version("1.8.0_361")?;

    assert_eq!(major, 1);
    assert_eq!(minor, 8);

    Ok(())
}

#[test]
fn check_java_greater_than_8() -> Result<(), JavaDomainError> {
    let (major, minor) = extract_java_major_minor_version("17")?;

    assert_eq!(major, 1);
    assert_eq!(minor, 17);

    Ok(())
}
