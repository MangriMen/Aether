/**
 * Describes the declarative capabilities of a plugin, such as supported importers.
 */
export interface PluginCapabilities {
  /**
   * List of supported modpack importers provided by the plugin.
   */
  importers: ImporterCapability[];
}

export interface ImporterCapability {
  /**
   * Unique identifier for the importer (lowercase, kebab/underscore allowed).
   */
  id: string;
  /**
   * Display name of the importer.
   */
  name: string;
  /**
   * Optional description of what this importer does.
   */
  description?: string;
  /**
   * Optional icon file name or URL for the importer.
   */
  icon?: string;
  /**
   * Optional field label for the importer
   */
  fieldLabel?: string;
}
