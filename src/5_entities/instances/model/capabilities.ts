export interface CapabilityMetadata {
  /** Identifier (lowercase, kebab/underscore allowed). */
  id: string;
  /** Display name. */
  name: string;
  /** Optional description. */
  description?: string;
  /** Optional icon file name or URL. */
  icon?: string;
}

export interface ImporterCapabilityMetadata extends CapabilityMetadata {
  /** Optional field label for the importer. */
  fieldLabel?: string;
}

// eslint-disable-next-line @typescript-eslint/no-empty-object-type
export interface UpdaterCapabilityMetadata extends CapabilityMetadata {}

export interface ContentProviderCapabilityMetadata extends CapabilityMetadata {
  providerDataContentIdField: string;
}
