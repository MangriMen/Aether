use extism::{Function, UserData, ValType, PTR};

use super::{features, PluginContext};

pub fn get_host_functions(plugin_id: &str) -> Vec<Function> {
    let context = PluginContext {
        id: plugin_id.to_string(),
    };

    [
        get_core_host_functions,
        get_java_host_functions,
        get_instance_host_functions,
    ]
    .iter()
    .flat_map(|func| func(&context))
    .collect()
}

pub fn get_core_host_functions(context: &PluginContext) -> Vec<Function> {
    vec![
        Function::new(
            "log",
            [ValType::I64, PTR],
            [],
            UserData::new(context.clone()),
            features::log,
        ),
        Function::new(
            "get_id",
            [],
            [PTR],
            UserData::new(context.clone()),
            features::get_id,
        ),
        Function::new(
            "run_command",
            [PTR],
            [PTR],
            UserData::new(context.clone()),
            features::run_command,
        ),
    ]
}

pub fn get_java_host_functions(context: &PluginContext) -> Vec<Function> {
    vec![
        Function::new(
            "get_java",
            [ValType::I64],
            [PTR],
            UserData::new(context.clone()),
            features::get_java,
        ),
        Function::new(
            "install_java",
            [ValType::I64],
            [PTR],
            UserData::new(context.clone()),
            features::install_java,
        ),
    ]
}

pub fn get_instance_host_functions(context: &PluginContext) -> Vec<Function> {
    vec![
        Function::new(
            "instance_get_dir",
            [PTR],
            [PTR],
            UserData::new(context.clone()),
            features::instance_get_dir,
        ),
        Function::new(
            "instance_plugin_get_dir",
            [PTR],
            [PTR],
            UserData::new(context.clone()),
            features::instance_plugin_get_dir,
        ),
        Function::new(
            "instance_create",
            [PTR],
            [PTR],
            UserData::new(context.clone()),
            features::instance_create,
        ),
        Function::new(
            "list_content",
            [PTR],
            [PTR],
            UserData::new(context.clone()),
            features::list_content,
        ),
        Function::new(
            "enable_contents",
            [PTR, PTR],
            [PTR],
            UserData::new(context.clone()),
            features::enable_contents,
        ),
        Function::new(
            "disable_contents",
            [PTR, PTR],
            [PTR],
            UserData::new(context.clone()),
            features::disable_contents,
        ),
    ]
}
