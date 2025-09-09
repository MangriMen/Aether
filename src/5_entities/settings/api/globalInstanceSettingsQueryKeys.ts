export const GLOBAL_INSTANCE_SETTINGS_QUERY_KEYS = {
  SELF: ['global_instance_settings'],
  GET: () => [...GLOBAL_INSTANCE_SETTINGS_QUERY_KEYS.SELF, 'get'],
};
