import type { ContentProviderCapabilityMetadata } from '@/features/instance';
import type { CapabilityEntry } from '@/features/plugins';
import type { Option } from '@/shared/ui-kit';

export type ContentProviderEntry =
  CapabilityEntry<ContentProviderCapabilityMetadata>;

export const contentProviderToOption = <T extends ContentProviderEntry>(
  value: T,
): Option<T> => ({
  name: value.capability.name,
  value,
});

export function contentProvidersToOptions<T extends ContentProviderEntry>(
  values: T[],
): Option<T>[];

export function contentProvidersToOptions(values: undefined): undefined;

export function contentProvidersToOptions<T extends ContentProviderEntry>(
  values: T[] | undefined,
): Option<T>[] | undefined;

export function contentProvidersToOptions<T extends ContentProviderEntry>(
  values: T[] | undefined,
) {
  if (!values) {
    return undefined;
  }

  return values.map(contentProviderToOption);
}
