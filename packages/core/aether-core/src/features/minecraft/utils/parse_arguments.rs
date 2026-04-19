use crate::features::minecraft::{MinecraftDomainError, TEMPORARY_REPLACE_CHAR, vanilla};

use super::parse_rules;

pub fn parse_arguments<F>(
    arguments: &[vanilla::Argument],
    parsed_arguments: &mut Vec<String>,
    parse_function: F,
    java_arch: &str,
) -> Result<(), MinecraftDomainError>
where
    F: Fn(&str) -> Result<String, MinecraftDomainError>,
{
    for argument in arguments {
        match argument {
            vanilla::Argument::Normal(arg) => {
                let parsed = parse_function(&arg.replace(' ', TEMPORARY_REPLACE_CHAR))?;
                for arg in parsed.split(TEMPORARY_REPLACE_CHAR) {
                    parsed_arguments.push(arg.to_string());
                }
            }
            vanilla::Argument::Ruled { rules, value } => {
                if parse_rules(rules, java_arch, true) {
                    match value {
                        vanilla::ArgumentValue::Single(arg) => {
                            parsed_arguments
                                .push(parse_function(&arg.replace(' ', TEMPORARY_REPLACE_CHAR))?);
                        }
                        vanilla::ArgumentValue::Many(args) => {
                            for arg in args {
                                parsed_arguments.push(parse_function(
                                    &arg.replace(' ', TEMPORARY_REPLACE_CHAR),
                                )?);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
