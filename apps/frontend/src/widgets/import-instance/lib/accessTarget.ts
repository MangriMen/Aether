/** What a value the user gave a plugin (an importer's field) needs the plugin to have. */
export type PluginAccessTarget =
  { kind: 'host'; url: string } | { kind: 'folder'; filePath: string };

const HTTP_URL = /^https?:\/\//i;

/** `C:\…`, `C:/…`, `\\server\…`, `/…`. */
const ABSOLUTE_PATH = /^(?:[a-zA-Z]:[\\/]|\\\\|\/)/;

/**
 * An http(s) URL needs its host, an absolute path the folder of the file. Anything else is the
 * importer's own business: nothing is asked for it.
 */
export const pluginAccessTargetOf = (
  value: string,
): PluginAccessTarget | null => {
  if (HTTP_URL.test(value)) {
    return { kind: 'host', url: value };
  }

  if (ABSOLUTE_PATH.test(value)) {
    return { kind: 'folder', filePath: value };
  }

  return null;
};
