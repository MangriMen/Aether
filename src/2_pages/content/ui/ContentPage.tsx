import { useSearchParams, type RouteSectionProps } from '@solidjs/router';
import {
  createMemo,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import type { ContentFilters } from '@/entities/instances';

import { useContentProviders, useInstance } from '@/entities/instances';
import { Separator } from '@/shared/ui';

import type { ContentFiltersLock } from '../model/contentFiltersLock';

import { getAvailableContentTypes, parseSearchParams } from '../lib';
import { getFiltersFromInstance } from '../lib/getFiltersFromInstance';
import { contentProvidersToOptions } from '../model';
import { ContentBrowser } from './ContentBrowser';
import { InstanceInfo } from './InstanceInfo';

export type ContentPageProps = ComponentProps<'div'> & RouteSectionProps;

export const ContentPage: Component<ContentPageProps> = (props) => {
  const [_, others] = splitProps(props, [
    'params',
    'location',
    'data',
    'children',
  ]);

  const [searchParams] = useSearchParams();

  const pageSearchParams = createMemo(() => parseSearchParams(searchParams));

  const instance = useInstance(() => pageSearchParams().instanceId);

  const contentProviders = useContentProviders();

  const contentProvidersOptions = createMemo(
    () => contentProvidersToOptions(contentProviders.data) ?? [],
  );

  const availableContentTypes = createMemo(() =>
    getAvailableContentTypes(instance.data, !!pageSearchParams().instanceId),
  );

  const filtersData = createMemo<{
    filters?: ContentFilters;
    filtersLock?: ContentFiltersLock;
  }>(() => {
    const filtersFromInstance = getFiltersFromInstance(instance.data);
    return filtersFromInstance;
  });

  return (
    <div class='flex size-full flex-col gap-2 p-4' {...others}>
      <Show when={instance.data}>
        {(instance) => (
          <>
            <InstanceInfo instance={instance()} />
            <Separator />
          </>
        )}
      </Show>
      <ContentBrowser
        instance={instance.data}
        providers={contentProvidersOptions()}
        types={availableContentTypes()}
        filters={filtersData().filters}
        filtersLock={filtersData().filtersLock}
      />
    </div>
  );
};
