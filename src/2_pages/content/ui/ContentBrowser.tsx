import { debounce } from '@solid-primitives/scheduled';
import {
  type Component,
  type ComponentProps,
  createEffect,
  createMemo,
  createSignal,
  For,
  Match,
  splitProps,
  Switch,
} from 'solid-js';

import type {
  ContentItemExtended,
  ContentRequest,
  Instance,
} from '@/entities/instances';
import type { Option } from '@/shared/model';

import {
  CONTENT_TYPE_TO_TITLE,
  CONTENT_TYPES,
  ContentType,
  useContentByProvider,
  useInstanceContents,
  useMetadataFieldToCheckInstalled,
} from '@/entities/instances';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedSelect, Tabs, TabsList, TabsTrigger } from '@/shared/ui';

import { ContentFilters } from './ContentFilters';
import { ContentList } from './ContentList';
import { ContentListSkeleton } from './ContentListSkeleton';

export type ContentBrowserProps = {
  contentTypes?: ContentType[];
  instance: Instance;
  providers: Option<string>[];
} & ComponentProps<'div'>;

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

  const availableContentTabs = createMemo(
    () => local.contentTypes ?? CONTENT_TYPES,
  );

  const [searchQuery, setSearchQuery] = createSignal<string | undefined>();
  const [provider, setProvider] = createSignal<null | Option<string>>(null);
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
  const handleSetProvider = (provider: null | Option<string>) => {
    if (provider) {
      setProvider(provider);
    }
  };

  const contentRequestPayload = createMemo((): ContentRequest | undefined => {
    const currentProvider = provider();
    if (!currentProvider) {
      return;
    }

    return {
      contentType: contentType(),
      gameVersions: [local.instance.gameVersion],
      loader: local.instance.loader,
      page: page(),
      pageSize: pageSize(),
      provider: currentProvider.value,
      query: searchQuery(),
    };
  });

  const content = useContentByProvider(() => contentRequestPayload());

  const instanceContent = useInstanceContents(() => local.instance.id);

  const instanceContentArray = createMemo(() =>
    instanceContent.data ? Object.values(instanceContent.data) : undefined,
  );

  const compareMetadataField = useMetadataFieldToCheckInstalled(
    () => provider()?.value,
  );

  const instanceContentFields = createMemo(() => {
    if (!compareMetadataField.data) {
      return [];
    }
    const providerValue = provider()?.value;
    if (!providerValue) {
      return [];
    }

    return instanceContentArray()?.map(
      (item) => item.update?.[providerValue]?.[compareMetadataField.data],
    );
  });

  const [newlyInstalled, setNewlyInstalled] = createSignal<
    NonNullable<ContentItemExtended['providerData']>[string][]
  >([]);

  const items = createMemo(() =>
    content.data?.items.map<ContentItemExtended>((item) => {
      const field = compareMetadataField.data;

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
    if (!compareMetadataField.data) {
      return;
    }

    const data = providerData?.[compareMetadataField.data];
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
        <Tabs onChange={setContentType}>
          <TabsList>
            <For each={availableContentTabs()}>
              {(contentType) => (
                <TabsTrigger value={contentType}>
                  {t(`content.${CONTENT_TYPE_TO_TITLE[contentType]}`)}
                </TabsTrigger>
              )}
            </For>
          </TabsList>
        </Tabs>
        <div class='flex items-center gap-2'>
          <span class='text-muted-foreground'>{t('content.provider')}:</span>
          <CombinedSelect
            onChange={handleSetProvider}
            options={local.providers}
            optionTextValue='name'
            optionValue='value'
            title='Provider'
            value={provider()}
          />
        </div>
      </div>
      <ContentFilters
        contentType={contentType()}
        currentPage={page()}
        loading={!content.data?.items.length}
        onPageChange={handlePageChange}
        onPageSizeChange={handlePageSizeChange}
        onSearch={handleSearch}
        pageCount={content.data?.pageCount ?? 10}
        pageSize={pageSize()}
      />
      <Switch>
        <Match when={content.isLoading}>
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
              gameVersion={local.instance.gameVersion}
              instanceId={local.instance.id}
              items={items() ?? []}
              loader={local.instance.loader}
              onInstalled={handleOnInstalled}
              provider={provider()?.value}
            />
          )}
        </Match>
      </Switch>
    </div>
  );
};
