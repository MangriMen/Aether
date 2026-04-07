export const DEFAULT_INSTANCE_SETTINGS_QUERY_KEYS = {
  SELF: ['default_instance_settings'],
  GET: () => [...DEFAULT_INSTANCE_SETTINGS_QUERY_KEYS.SELF, 'get'],
};
