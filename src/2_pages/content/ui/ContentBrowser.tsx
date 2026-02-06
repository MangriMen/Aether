import { debounce } from '@solid-primitives/scheduled';
import {
  createEffect,
  createMemo,
  createSignal,
  Match,
  splitProps,
  Switch,
  type Component,
  type ComponentProps,
} from 'solid-js';

import type {
  ContentItemExtended,
  ContentRequest,
  Instance,
} from '@/entities/instances';
import type { ContentProviderCapabilityMetadata } from '@/entities/instances/model/capabilities';
import type { CapabilityEntry, Option } from '@/shared/model';

import {
  CONTENT_TYPES,
  ContentType,
  useSearchContent,
  useInstanceContents,
} from '@/entities/instances';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedSelect } from '@/shared/ui';

import { ContentFilters } from './ContentFilters';
import { ContentList } from './ContentList';
import { ContentListSkeleton } from './ContentListSkeleton';
import { ContentTypeTabs } from './ContentTypeTabs';

export type ContentBrowserProps = ComponentProps<'div'> & {
  providers: Option<CapabilityEntry<ContentProviderCapabilityMetadata>>[];
  instance?: Instance;
  contentTypes?: ContentType[];
};

export const ContentBrowser: Component<ContentBrowserProps> = (props) => {
  const [local, others] = splitProps(props, [
    'providers',
    'instance',
    'contentTypes',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const [contentType, setContentType] = createSignal<ContentType>(
    ContentType.Mod,
  );

  createEffect(() => {
    setContentType(local.instance ? ContentType.Mod : ContentType.Modpack);
  });

  const availableContentTabs = createMemo(
    () => local.contentTypes ?? CONTENT_TYPES,
  );

  const [searchQuery, setSearchQuery] = createSignal<string | undefined>();
  const [provider, setProvider] = createSignal<Option<
    CapabilityEntry<ContentProviderCapabilityMetadata>
  > | null>(null);
  const [page, setPage] = createSignal(1);
  const [pageSize, setPageSize] = createSignal(20);

  createEffect(() => {
    if (local.providers.length) {
      setProvider(local.providers[0]);
    }
  });

  const handleSearch = debounce((query: string) => {
    if (query.trim()) {
      setPage(1);
    }
    setSearchQuery(query);
  }, 300);
  const handlePageChange = (page: number) => setPage(page);
  const handlePageSizeChange = (pageSize: number) => setPageSize(pageSize);
  const handleSetProvider = (
    provider: Option<CapabilityEntry<ContentProviderCapabilityMetadata>> | null,
  ) => {
    if (provider) {
      setProvider(provider);
    }
  };

  const contentRequestPayload = createMemo((): ContentRequest | undefined => {
    const currentProvider = provider();
    if (!currentProvider) {
      return;
    }

    const gameVersions = local.instance
      ? [local.instance.gameVersion]
      : undefined;

    const loader = local.instance?.loader;

    return {
      contentType: contentType(),
      provider: currentProvider.value.capability.id,
      page: page(),
      pageSize: pageSize(),
      query: searchQuery(),
      gameVersions,
      loader,
    };
  });

  const content = useSearchContent(() => contentRequestPayload());

  const instanceContent = useInstanceContents(() => local.instance?.id);

  const instanceContentArray = createMemo(() =>
    instanceContent.data ? Object.values(instanceContent.data) : undefined,
  );

  const compareMetadataField = createMemo(
    () => provider()?.value.capability.providerDataContentIdField,
  );

  const instanceContentFields = createMemo(() => {
    const field = compareMetadataField();
    if (!field) {
      return [];
    }

    const providerValue = provider()?.value.capability.id;
    if (!providerValue) {
      return [];
    }

    return instanceContentArray()?.map(
      (item) => item.update?.[providerValue]?.[field],
    );
  });

  const [newlyInstalled, setNewlyInstalled] = createSignal<
    NonNullable<ContentItemExtended['providerData']>[string][]
  >([]);

  const items = createMemo(() =>
    content.data?.items.map<ContentItemExtended>((item) => {
      const field = compareMetadataField();

      const contentItemField = field ? item.providerData?.[field] : undefined;
      const installed =
        contentItemField && typeof contentItemField === 'string'
          ? Boolean(newlyInstalled().includes(contentItemField)) ||
            Boolean(instanceContentFields()?.includes(contentItemField))
          : false;

      return {
        ...item,
        installed,
      };
    }),
  );

  const handleOnInstalled = (
    providerData: ContentItemExtended['providerData'],
  ) => {
    const field = compareMetadataField();
    if (!field) {
      return;
    }

    const data = providerData?.[field];
    if (!data) {
      return;
    }

    setNewlyInstalled((prev) => [...prev, data]);
  };

  return (
    <div
      class={cn('flex flex-col grow gap-2 overflow-hidden p-1', local.class)}
      {...others}
    >
      <div class='flex justify-between gap-2'>
        <ContentTypeTabs
          onChange={setContentType}
          items={availableContentTabs()}
        />
        <div class='flex items-center gap-2'>
          <span class='text-muted-foreground'>{t('content.provider')}:</span>
          <CombinedSelect
            options={local.providers}
            value={provider()}
            onChange={handleSetProvider}
            optionValue='value'
            optionTextValue='name'
            title='Provider'
          />
        </div>
      </div>
      <ContentFilters
        pageSize={pageSize()}
        pageCount={content.data?.pageCount}
        currentPage={page()}
        onSearch={handleSearch}
        onPageChange={handlePageChange}
        onPageSizeChange={handlePageSizeChange}
        contentType={contentType()}
        loading={content.isFetching}
      />
      <Switch>
        <Match when={content.isFetching}>
          <ContentListSkeleton />
        </Match>
        <Match when={content.isError}>
          <span class='flex grow items-center justify-center text-xl font-medium text-muted-foreground'>
            {t('content.providerErrorOrNotFound')}
          </span>
        </Match>
        <Match when={items()}>
          {(items) => (
            <ContentList
              items={items()}
              instanceId={local.instance?.id}
              gameVersion={local.instance?.gameVersion}
              loader={local.instance?.loader}
              provider={provider()?.value.capability.id}
              onInstalled={handleOnInstalled}
            />
          )}
        </Match>
      </Switch>
    </div>
  );
};
