import type { CapabilityEntryDto } from '@/entities/instances/@x/plugins';
import type { ImporterCapabilityMetadataDto } from '@/entities/instances/@x/plugins';

import type { UpdaterCapability } from './pluginCapabilities';

export type CapabilityEntry<C> = CapabilityEntryDto<C>;

export type ImporterCapabilityEntry =
  CapabilityEntry<ImporterCapabilityMetadataDto>;
export type UpdaterCapabilityEntry = CapabilityEntry<UpdaterCapability>;
