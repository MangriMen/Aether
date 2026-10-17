import type {
  CapabilityEntryDto,
  ImporterCapabilityMetadataDto,
} from '@/platform/bindings/instance';

import type { UpdaterCapability } from './pluginCapabilities';

export type CapabilityEntry<C> = CapabilityEntryDto<C>;

export type ImporterCapabilityMetadata = ImporterCapabilityMetadataDto;

export type ImporterCapabilityEntry =
  CapabilityEntry<ImporterCapabilityMetadataDto>;
/** @public Dto alias, kept while unused (ARCHITECTURE.md, Checks). */
export type UpdaterCapabilityEntry = CapabilityEntry<UpdaterCapability>;
