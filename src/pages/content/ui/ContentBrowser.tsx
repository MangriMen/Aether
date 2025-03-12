import {
  CONTENT_TYPES,
  ContentType,
  installContent,
} from '@/entities/instances';

import type {
  ContentItem,
  ContentRequest,
  Instance,
} from '@/entities/instances';

import { cn } from '@/shared/lib';
import type { Option } from '@/shared/model';
import { useTranslate } from '@/shared/model';
import { CombinedSelect, Tabs, TabsList, TabsTrigger } from '@/shared/ui';
import {
  createEffect,
  createMemo,
  createSignal,
  For,
  Match,
  splitProps,
  Switch,
  type Component,
  type ComponentProps,
} from 'solid-js';
import { ContentFilters } from './ContentFilters';
import { ContentList } from './ContentList';
import { CONTENT_TYPE_TO_TITLE } from '../model';
import { debounce } from '@solid-primitives/scheduled';
import { ContentListSkeleton } from './ContentListSkeleton';
import { useContent } from '../lib';

export type ContentBrowserProps = ComponentProps<'div'> & {
  providers: Option<string>[];
  instance: Instance;
  availableContent?: ContentType[];
};

export const ContentBrowser: Component<ContentBrowserProps> = (props) => {
  const [local, others] = splitProps(props, [
    'providers',
    'instance',
    'availableContent',
    'class',
  ]);

  const [{ t }] = useTranslate();

  const [contentType, setContentType] = createSignal<ContentType>(
    ContentType.Mod,
  );

  const availableContentTabs = createMemo(
    () => local.availableContent ?? CONTENT_TYPES,
  );

  const [searchQuery, setSearchQuery] = createSignal<string | undefined>();
  const [provider, setProvider] = createSignal<Option<string> | null>(null);
  const [page, setPage] = createSignal(1);
  const [pageSize, setPageSize] = createSignal(20);

  createEffect(() => {
    if (local.providers.length) {
      setProvider(local.providers[0]);
    }
  });

  const handleSearch = debounce(setSearchQuery, 300);
  const handlePageChange = (page: number) => setPage(page);
  const handlePageSizeChange = (pageSize: number) => setPageSize(pageSize);
  const handleSetProvider = (provider: Option<string> | null) => {
    if (provider) {
      setProvider(provider);
    }
  };

  const contentRequestPayload = createMemo((): ContentRequest | null => {
    const currentProvider = provider();
    if (!currentProvider) {
      return null;
    }

    return {
      contentType: contentType(),
      provider: currentProvider.value,
      page: page(),
      pageSize: pageSize(),
      query: searchQuery(),
    };
  });

  const [content, { isLoading: isContentLoading }] = useContent(() =>
    contentRequestPayload(),
  );

  const handleInstallContent = (content_item: ContentItem) => {
    const provider = content().data?.provider;
    if (!provider) {
      return;
    }

    installContent(local.instance.id, content_item, provider);
  };

  return (
    <div
      class={cn('flex flex-col grow gap-2 overflow-hidden p-1', local.class)}
      {...others}
    >
      <div class='flex gap-2'>
        <CombinedSelect
          options={local.providers}
          value={provider()}
          onChange={handleSetProvider}
          optionValue='value'
          optionTextValue='name'
        />
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
      </div>
      <ContentFilters
        pageSize={pageSize()}
        pageCount={content()?.data?.pageCount ?? 0}
        currentPage={page()}
        onSearch={handleSearch}
        onPageChange={handlePageChange}
        onPageSizeChange={handlePageSizeChange}
        contentType={contentType()}
      />
      <Switch>
        <Match when={isContentLoading()}>
          <ContentListSkeleton />
        </Match>
        <Match when={content().error}>
          <span class='flex grow items-center justify-center text-xl font-medium text-muted-foreground'>
            {t('content.providerErrorOrNotFound')}
          </span>
        </Match>
        <Match when={content().data}>
          {(contentRequest) => (
            <ContentList
              items={contentRequest().items ?? []}
              onInstall={handleInstallContent}
            />
          )}
        </Match>
      </Switch>
    </div>
  );
};
