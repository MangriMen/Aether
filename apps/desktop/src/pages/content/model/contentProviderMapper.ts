import type { ContentProviderCapabilityMetadata } from '../../../entities/instances';
import type { CapabilityEntry, Option } from '../../../shared/model';

export type ContentProviderEntry =
  CapabilityEntry<ContentProviderCapabilityMetadata>;

export const contentProviderToOption = <T extends ContentProviderEntry>(
  value: T,
): Option<T> => ({
  name: value.capability.name,
  value: value,
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
  if (!values) return undefined;

  return values.map(contentProviderToOption);
}
