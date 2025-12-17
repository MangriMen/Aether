import {
  createMemo,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { CONTENT_TYPE_TO_TITLE, type ContentType } from '@/entities/instances';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import {
  CombinedPagination,
  CombinedTextField,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Skeleton,
} from '@/shared/ui';

export type ContentFiltersProps = ComponentProps<'div'> & {
  pageCount: number;
  pageSize: number;
  currentPage: number;
  onSearch: (query: string) => void;
  onPageChange: (page: number) => void;
  onPageSizeChange: (pageSize: number) => void;
  contentType?: ContentType;
  loading?: boolean;
};

const PER_PAGE_OPTIONS = [5, 10, 20, 30, 40, 50];

export const ContentFilters: Component<ContentFiltersProps> = (props) => {
  const [local, others] = splitProps(props, [
    'pageCount',
    'pageSize',
    'currentPage',
    'onSearch',
    'onPageChange',
    'onPageSizeChange',
    'contentType',
    'loading',
    'class',
  ]);

  const [{ t }] = useTranslation();

  const searchPlaceholder = createMemo(() => {
    const searchText = t('common.search');
    const contentTypeText = local.contentType
      ? t(`content.${CONTENT_TYPE_TO_TITLE[local.contentType]}`)
      : t('content.title');

    return `${searchText} ${contentTypeText}`;
  });

  return (
    <div class={cn('flex gap-2 justify-between', local.class)} {...others}>
      <CombinedTextField
        class='w-full'
        name='search'
        inputProps={{
          type: 'text',
          placeholder: searchPlaceholder(),
        }}
        onChange={local.onSearch}
      />
      <div class='flex justify-between gap-2'>
        <Select
          options={PER_PAGE_OPTIONS}
          value={local.pageSize}
          onChange={(value) => {
            if (value) {
              local.onPageSizeChange(value);
            }
          }}
          itemComponent={(props) => (
            <SelectItem item={props.item}>{props.item.textValue}</SelectItem>
          )}
        >
          <SelectTrigger class='gap-1.5 whitespace-nowrap'>
            <SelectValue<number>>
              {(state) => `${t('common.perPage')}: ${state.selectedOption()}`}
            </SelectValue>
          </SelectTrigger>
          <SelectContent />
        </Select>

        <Show
          when={!local.loading}
          fallback={<Skeleton width={200} radius={6} class='bg-secondary' />}
        >
          <Show when={local.pageCount > 1}>
            <CombinedPagination
              siblingCount={1}
              count={local.pageCount}
              page={local.currentPage}
              onPageChange={local.onPageChange}
              disabled={local.loading}
            />
          </Show>
        </Show>
      </div>
    </div>
  );
};
