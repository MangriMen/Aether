use tauri_build::{DefaultPermissionRule, InlinedPlugin};

fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .codegen(tauri_build::CodegenContext::new())
            .plugin(
                "instance",
                InlinedPlugin::new()
                    .commands(&[
                        "instance_create",
                        "instance_list",
                        "instance_get",
                        "instance_edit",
                        "instance_remove",
                        "instance_get_contents",
                        "instance_toggle_disable_content",
                        "instance_remove_content",
                        "instance_launch",
                        "instance_stop",
                        "instance_import",
                        "instance_get_import_handlers",
                    ])
                    .default_permission(DefaultPermissionRule::AllowAllCommands),
            )
            .plugin(
                "process",
                InlinedPlugin::new()
                    .commands(&["process_list", "process_get_by_instance_id"])
                    .default_permission(DefaultPermissionRule::AllowAllCommands),
            ),
    )
    .expect("Failed to run tauri-build");
}
