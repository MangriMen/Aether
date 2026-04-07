use std::path::Path;

use uuid::Uuid;

use crate::{
    features::{
        auth::Credentials,
        minecraft::{utils::parse_arguments, MinecraftDomainError},
        settings::WindowSize,
    },
    shared::canonicalize,
};

// Replaces the space separator with a newline character, as to not split the arguments
const TEMPORARY_REPLACE_CHAR: &str = "\n";

#[allow(clippy::too_many_arguments)]
pub fn get_minecraft_arguments(
    arguments: Option<&[daedalus::minecraft::Argument]>,
    legacy_arguments: Option<&str>,
    credentials: &Credentials,
    version: &str,
    asset_index_name: &str,
    game_directory: &Path,
    assets_directory: &Path,
    version_type: &daedalus::minecraft::VersionType,
    resolution: WindowSize,
    java_arch: &str,
) -> Result<Vec<String>, MinecraftDomainError> {
    if let Some(arguments) = arguments {
        let mut parsed_arguments = Vec::new();

        parse_arguments(
            arguments,
            &mut parsed_arguments,
            |arg| {
                replace_placeholders_in_argument_string(
                    arg,
                    credentials.access_token(),
                    credentials.username.as_ref(),
                    credentials.id(),
                    version,
                    asset_index_name,
                    game_directory,
                    assets_directory,
                    version_type,
                    resolution,
                )
            },
            java_arch,
        )?;

        Ok(parsed_arguments)
    } else if let Some(legacy_arguments) = legacy_arguments {
        let mut parsed_arguments = Vec::new();
        for x in legacy_arguments.split(' ') {
            parsed_arguments.push(replace_placeholders_in_argument_string(
                &x.replace(' ', TEMPORARY_REPLACE_CHAR),
                credentials.access_token(),
                credentials.username.as_ref(),
                credentials.id(),
                version,
                asset_index_name,
                game_directory,
                assets_directory,
                version_type,
                resolution,
            )?);
        }
        Ok(parsed_arguments)
    } else {
        Ok(Vec::new())
    }
}

#[allow(clippy::too_many_arguments)]
fn replace_placeholders_in_argument_string(
    argument: &str,
    access_token: &str,
    username: &str,
    uuid: Uuid,
    version: &str,
    asset_index_name: &str,
    game_directory: &Path,
    assets_directory: &Path,
    version_type: &daedalus::minecraft::VersionType,
    resolution: WindowSize,
) -> Result<String, MinecraftDomainError> {
    fn resolve_path(path: &Path, name: &str) -> Result<String, MinecraftDomainError> {
        Ok(canonicalize(path)
            .map_err(|_| MinecraftDomainError::PathNotFound {
                path: path.to_path_buf(),
                entity_type: name.to_owned(),
            })?
            .to_string_lossy()
            .to_string())
    }

    let uuid_str = uuid.simple().to_string();
    let game_dir_str = resolve_path(game_directory, "game")?;
    let assets_dir_str = resolve_path(assets_directory, "assets")?;
    let resolution_width = resolution.0.to_string();
    let resolution_height = resolution.1.to_string();

    Ok(argument
        .replace("${accessToken}", access_token)
        .replace("${auth_access_token}", access_token)
        .replace("${auth_session}", access_token)
        .replace("${auth_player_name}", username)
        .replace("${auth_xuid}", "0")
        .replace("${auth_uuid}", &uuid_str)
        .replace("${uuid}", &uuid_str)
        .replace("${clientid}", "c4502edb-87c6-40cb-b595-64a280cf8906")
        .replace("${user_properties}", "{}")
        .replace("${user_type}", "msa")
        .replace("${version_name}", version)
        .replace("${assets_index_name}", asset_index_name)
        .replace("${game_directory}", &game_dir_str)
        .replace("${assets_root}", &assets_dir_str)
        .replace("${game_assets}", &assets_dir_str)
        .replace("${version_type}", version_type.as_str())
        .replace("${resolution_width}", &resolution_width)
        .replace("${resolution_height}", &resolution_height))
}
