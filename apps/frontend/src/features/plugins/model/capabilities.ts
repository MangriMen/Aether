import type {
  CapabilityEntryDto,
  ImporterCapabilityMetadataDto,
} from '@/platform/bindings/instance';

import type { UpdaterCapability } from './pluginCapabilities';

export type CapabilityEntry<C> = CapabilityEntryDto<C>;

export type ImporterCapabilityMetadata = ImporterCapabilityMetadataDto;

export type ImporterCapabilityEntry =
  CapabilityEntry<ImporterCapabilityMetadataDto>;
export type UpdaterCapabilityEntry = CapabilityEntry<UpdaterCapability>;
