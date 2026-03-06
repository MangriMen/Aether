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

import {
  getAvailableContentTypes,
  getFiltersFromSearchParams,
  parseSearchParams,
  getFiltersFromInstance,
  mergeContentFilters,
} from '../lib';
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

  const [searchParams, setSearchParams] = useSearchParams();

  const params = createMemo(() => parseSearchParams(searchParams));

  const instance = useInstance(() => params().instanceId);

  const isInstanceContentPage = createMemo(() => !!params().instanceId);

  const availableContentTypes = createMemo(() =>
    getAvailableContentTypes(instance.data, isInstanceContentPage()),
  );

  const contentProviders = useContentProviders();

  const contentProvidersOptions = createMemo(
    () => contentProvidersToOptions(contentProviders.data) ?? [],
  );

  const filtersData = createMemo(() =>
    mergeContentFilters(
      getFiltersFromInstance(instance.data),
      getFiltersFromSearchParams(params()),
    ),
  );

  const handleFiltersChange = (filters: ContentFilters) => {
    setSearchParams({
      page: filters.page ? encodeURIComponent(filters.page) : undefined,
      pageSize: filters.pageSize
        ? encodeURIComponent(filters.pageSize)
        : undefined,
      query: filters.query ? encodeURIComponent(filters.query) : undefined,
      provider: filters.provider
        ? encodeURIComponent(filters.provider)
        : undefined,
      contentType: filters.contentType
        ? encodeURIComponent(filters.contentType)
        : undefined,
      gameVersions: filters.gameVersions
        ? filters.gameVersions.map((x) => encodeURIComponent(x))
        : undefined,
      loaders: filters.loaders
        ? filters.loaders.map((x) => encodeURIComponent(x))
        : undefined,
    });
  };

  return (
    <div class='flex size-full flex-col gap-2 p-4' {...others}>
      <Show when={isInstanceContentPage()}>
        <>
          <InstanceInfo instance={instance.data} />
          <Separator />
        </>
      </Show>
      <ContentBrowser
        instance={instance.data}
        providers={contentProvidersOptions()}
        types={availableContentTypes()}
        filters={filtersData().filters}
        filtersLock={filtersData().filtersLock}
        onFiltersChange={handleFiltersChange}
      />
    </div>
  );
};
