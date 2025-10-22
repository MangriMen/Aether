/**
 * Describes the declarative capabilities of a plugin, such as supported importers.
 */
export interface PluginCapabilities {
  /**
   * List of supported modpack importers provided by the plugin.
   */
  importers: Importer[];
}

export interface Importer {
  /**
   * Optional description of what this importer does.
   */
  description?: string;
  /**
   * Optional icon file name or URL for the importer.
   */
  icon?: string;
  /**
   * Unique identifier for the importer (lowercase, kebab/underscore allowed).
   */
  id: string;
  /**
   * Display name of the importer.
   */
  name: string;
}
