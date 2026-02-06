import type { ContentProviderCapabilityMetadata } from '@/entities/instances';
import type { CapabilityEntry, Option } from '@/shared/model';

export const contentProviderToOption = <
  T extends CapabilityEntry<ContentProviderCapabilityMetadata>,
>(
  value: T,
): Option<T> => ({
  name: value.capability.name,
  value: value,
});

export function contentProvidersToOptions<
  T extends CapabilityEntry<ContentProviderCapabilityMetadata>,
>(values: T[]): Option<T>[];

export function contentProvidersToOptions(values: undefined): undefined;

export function contentProvidersToOptions<
  T extends CapabilityEntry<ContentProviderCapabilityMetadata>,
>(values: T[] | undefined): Option<T>[] | undefined;

export function contentProvidersToOptions<
  T extends CapabilityEntry<ContentProviderCapabilityMetadata>,
>(values: T[] | undefined) {
  if (!values) return undefined;

  return values.map(contentProviderToOption);
}
