import {
  createForm,
  getValue,
  getValues,
  reset,
  setValue,
  zodForm,
} from '@modular-forms/solid';
import { debounce } from '@solid-primitives/scheduled';
import {
  createEffect,
  createMemo,
  createSignal,
  Match,
  mergeProps,
  on,
  splitProps,
  Switch,
  type Component,
  type ComponentProps,
} from 'solid-js';

import type {
  ContentFilters,
  ContentSearchParams,
  Instance,
} from '@/entities/instances';
import type { ContentProviderCapabilityMetadata } from '@/entities/instances/model/capabilities';
import type { CapabilityEntry, Option } from '@/shared/model';

import { ContentType } from '@/entities/instances';
import { CONTENT_TYPES, useSearchContent } from '@/entities/instances';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedSelect } from '@/shared/ui';

import type { ContentFiltersLock } from '../model/contentFiltersLock';

import {
  ContentSearchSchema,
  type ContentSearchInputValues,
} from '../model/validation';
import { ContentContextProvider } from './ContentContextProvider';
import { ContentList } from './ContentList';
import { ContentListSkeleton } from './ContentListSkeleton';
import { ContentSearchCard } from './ContentSearchCard';
import { ContentTypeTabs } from './ContentTypeTabs';

export type ContentBrowserProps = Omit<ComponentProps<'form'>, 'onSubmit'> & {
  providers: Option<CapabilityEntry<ContentProviderCapabilityMetadata>>[];
  types?: ContentType[];
  instance?: Instance;
  filters?: ContentFilters;
  filtersLock?: ContentFiltersLock;
  onFiltersChange?: (filters: ContentFilters) => void;
};

export const ContentBrowser: Component<ContentBrowserProps> = (props) => {
  const [_local, others] = splitProps(props, [
    'providers',
    'instance',
    'types',
    'filters',
    'filtersLock',
    'onFiltersChange',
    'class',
  ]);

  const local = mergeProps(
    {
      types: CONTENT_TYPES,
    },
    _local,
  );

  const [{ t }] = useTranslation();

  const [form, { Form, Field }] = createForm<ContentSearchInputValues>({
    validate: zodForm(ContentSearchSchema),
    initialValues: {
      page: 1,
      pageSize: 20,
      provider: undefined,
      contentType: ContentType.Mod,
      gameVersions: undefined,
      loaders: undefined,
    },
  });

  const findProvider = (
    pluginId: string | undefined,
    capabilityId: string | undefined,
  ) => {
    return local.providers.find(
      (x) =>
        x.value.pluginId === pluginId && x.value.capability.id === capabilityId,
    );
  };

  createEffect(() => {
    const providerStr = local.filters?.provider?.split('_');
    const provider = findProvider(providerStr?.[0], providerStr?.[1])?.value;
    const firstProvider = local.providers[0];
    const defaultProvider = firstProvider
      ? {
          pluginId: firstProvider.value.pluginId,
          capabilityId: firstProvider.value.capability.id,
        }
      : undefined;

    reset(form, {
      initialValues: {
        page: local.filters?.page,
        pageSize: local.filters?.pageSize,
        provider: provider ?? defaultProvider,
        contentType: local.filters?.contentType ?? local.types[0],
        gameVersions: local.filters?.gameVersions,
        loaders: local.filters?.loaders,
      },
    });
  });

  createEffect(
    on([() => form, () => getValues(form)], ([_, values]) => {
      const filters = ContentSearchSchema.safeParse(values);

      if (filters.error) {
        return;
      }

      local.onFiltersChange?.(filters.data);
    }),
  );

  const contentType = createMemo(
    () => getValue(form, 'contentType') ?? ContentType.Mod,
  );
  const searchQuery = createMemo(() => getValue(form, 'query'));
  const handleChangeContentType = (type: ContentType) => {
    setValue(form, 'contentType', type, {
      shouldDirty: true,
      shouldTouched: true,
      shouldValidate: true,
    });
  };

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

  const handleSearch = (query: string) => {
    if (query.trim()) {
      setPage(1);
    }

    setValue(form, 'query', query);
  };

  const handlePageChange = (page: number) => setPage(page);
  const handlePageSizeChange = (pageSize: number) => setPageSize(pageSize);
  const handleSetProvider = (
    provider: Option<CapabilityEntry<ContentProviderCapabilityMetadata>> | null,
  ) => {
    if (provider) {
      setProvider(provider);
      setValue(form, 'provider.capabilityId', provider.value.capability.id);
      setValue(form, 'provider.pluginId', provider.value.pluginId);
    }
  };

  const [contentSearchParams, setContentSearchParams] = createSignal<
    ContentSearchParams | undefined
  >(undefined);

  const updateSearchParams = debounce((params: ContentSearchParams) => {
    setContentSearchParams(params);
  }, 300);

  createEffect(() => {
    const currentProvider = provider();
    if (!currentProvider) {
      return;
    }

    const dto = {
      contentType: contentType(),
      provider: currentProvider.value.capability.id,
      page: page(),
      pageSize: pageSize(),
      query: searchQuery(),
      gameVersions: local.filters?.gameVersions,
      loader: local.filters?.loaders?.[0],
    };

    updateSearchParams(dto);
  });

  const content = useSearchContent(() => contentSearchParams());

  return (
    <ContentContextProvider
      providerId={provider()?.value.capability.id}
      providerDataContentIdField={
        provider()?.value.capability.providerDataContentIdField
      }
      filters={local.filters}
      instanceId={local.instance?.id}
    >
      <Form
        class={cn('flex flex-col grow gap-2 overflow-hidden p-1', local.class)}
        {...others}
      >
        <div class='flex justify-between gap-2'>
          <Field name='contentType'>
            {(field) => (
              <ContentTypeTabs
                value={field.value}
                onChange={handleChangeContentType}
                items={local.types}
              />
            )}
          </Field>
          <div class='flex items-center gap-2'>
            <span class='text-muted-foreground'>{t('content.provider')}:</span>
            <Field name='provider.capabilityId'>
              {(capabilityId) => (
                <Field name='provider.pluginId'>
                  {(pluginId) => {
                    return (
                      <CombinedSelect
                        options={local.providers}
                        value={findProvider(pluginId.value, capabilityId.value)}
                        onChange={handleSetProvider}
                        optionValue='value'
                        optionTextValue='name'
                        title={t('content.provider')}
                      />
                    );
                  }}
                </Field>
              )}
            </Field>
          </div>
        </div>
        <ContentSearchCard
          pageSize={pageSize()}
          pageCount={content.data?.pageCount}
          currentPage={page()}
          searchQuery={form.internal.fields.query?.value.get() ?? ''}
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
          <Match when={content.data?.items}>
            {(items) => <ContentList items={items()} />}
          </Match>
        </Switch>
      </Form>
    </ContentContextProvider>
  );
};
