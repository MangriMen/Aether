import type { ContentProviderCapabilityMetadata } from '@/features/instance';
import type { CapabilityEntryDto } from '@/platform/bindings/instance';

export interface PlatformProvider {
  /** `pluginId#capabilityId`, the form the settings store. */
  id: string;
  name: string;
}

export interface PlatformProviders {
  platform: string;
  providers: PlatformProvider[];
}

export const groupProvidersByPlatform = (
  entries: CapabilityEntryDto<ContentProviderCapabilityMetadata>[],
): PlatformProviders[] => {
  const groups = new Map<string, PlatformProvider[]>();

  for (const { pluginId, capability } of entries) {
    if (!capability.platform) {
      continue;
    }

    const providers = groups.get(capability.platform) ?? [];
    providers.push({
      id: `${pluginId}#${capability.id}`,
      name: `${capability.name} (${pluginId})`,
    });
    groups.set(capability.platform, providers);
  }

  return [...groups].map(([platform, providers]) => ({ platform, providers }));
};
