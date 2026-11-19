/** What a value the user gave a plugin (an importer's field) needs the plugin to have. */
export type PluginAccessTarget =
  | { kind: 'host'; url: string }
  | { kind: 'folder'; filePath: string };

const HTTP_URL = /^https?:\/\//i;

/** `C:\…`, `C:/…`, `\\server\…`, `/…`. */
const ABSOLUTE_PATH = /^(?:[a-zA-Z]:[\\/]|\\\\|\/)/;

/** Four dotted numbers: how a URL writes back every IPv4 host it parses. */
const IPV4_HOSTNAME = /^\d{1,3}(?:\.\d{1,3}){3}$/;

/**
 * Whether the host of `url` is `localhost` itself or an address literal: a host the core can
 * tell is local from the URL alone, without resolving it. Whether a literal is local, the core
 * decides. A name under `localhost` is a DNS name like any other: it is asked for as one.
 */
export const namesLocalhostOrAddress = (url: string): boolean => {
  try {
    const { hostname } = new URL(url);

    return (
      hostname === 'localhost' ||
      hostname === 'localhost.' ||
      hostname.startsWith('[') ||
      IPV4_HOSTNAME.test(hostname)
    );
  } catch {
    return false;
  }
};

/**
 * An http(s) URL needs its host, an absolute path its folder (the picked one, or the one of the
 * picked file). Anything else is the importer's own business: nothing is asked for it.
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
