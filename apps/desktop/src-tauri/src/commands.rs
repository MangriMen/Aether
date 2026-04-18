pub const APPLICATION_PLUGIN_NAME: &str = "application";

pub const AUTH_PLUGIN_NAME: &str = "auth";
pub const EVENTS_PLUGIN_NAME: &str = "events";
pub const INSTANCE_PLUGIN_NAME: &str = "instance";
pub const MINECRAFT_PLUGIN_NAME: &str = "minecraft";
pub const PLUGIN_PLUGIN_NAME: &str = "plugin";
pub const PROCESS_PLUGIN_NAME: &str = "process";
pub const SETTINGS_PLUGIN_NAME: &str = "settings";
pub const UPDATE_PLUGIN_NAME: &str = "update";

macro_rules! application_commands {
    ($($tokens:tt)*) => {
        $($tokens)* [
            initialize_state,
            initialize_plugins,
        ]
    };
}

macro_rules! auth_commands {
    ($($tokens:tt)*) => {
        $($tokens)* [
            list_accounts,
            create_offline_account,
            change_account,
            logout,
        ]
    };
}

macro_rules! events_commands {
    ($($tokens:tt)*) => {
        $($tokens)* [
            list_progress_bars,
        ]
    };
}

macro_rules! instance_commands {
    ($($tokens:tt)*) => {
        $($tokens)* [
            create,
            import,
            list_importers,
            list,
            get,
            get_dir,
            install,
            update,
            edit,
            remove,
            launch,
            stop,
            import_contents,
            list_content,
            install_content,
            enable_contents,
            disable_contents,
            remove_contents,
            list_content_providers,
            search_content,
            check_compatibility,
            get_content,
            list_content_version,
        ]
    };
}

macro_rules! minecraft_commands {
    ($($tokens:tt)*) => {
        $($tokens)* [
            get_minecraft_version_manifest,
            get_loader_version_manifest,
        ]
    };
}

macro_rules! plugin_commands {
    ($($tokens:tt)*) => {
        $($tokens)* [
            import,
            sync,
            list,
            get,
            remove,
            enable,
            disable,
            call,
            get_settings,
            edit_settings,
            open_plugins_folder,
            get_api_version,
        ]
    };
}

macro_rules! process_commands {
    ($($tokens:tt)*) => {
        $($tokens)* [
            list,
            get_by_instance_id,
        ]
    };
}

macro_rules! settings_commands {
    ($($tokens:tt)*) => {
        $($tokens)* [
            get,
            edit,
            get_max_ram,
            get_default_instance_settings,
            edit_default_instance_settings,
            get_app_settings,
            edit_app_settings,
        ]
    };
}

macro_rules! update_commands {
    ($($tokens:tt)*) => {
        $($tokens)* [
            check_for_updates,
            install_update,
        ]
    };
}

pub(crate) use application_commands;

pub(crate) use auth_commands;
pub(crate) use events_commands;
pub(crate) use instance_commands;
pub(crate) use minecraft_commands;
pub(crate) use plugin_commands;
pub(crate) use process_commands;
pub(crate) use settings_commands;
pub(crate) use update_commands;
