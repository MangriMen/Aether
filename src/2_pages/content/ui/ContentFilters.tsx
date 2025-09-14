import {
  type Component,
  type ComponentProps,
  createMemo,
  Show,
  splitProps,
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

export type ContentFiltersProps = {
  contentType?: ContentType;
  currentPage: number;
  loading?: boolean;
  onPageChange: (page: number) => void;
  onPageSizeChange: (pageSize: number) => void;
  onSearch: (query: string) => void;
  pageCount: number;
  pageSize: number;
} & ComponentProps<'div'>;

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
        inputProps={{
          placeholder: searchPlaceholder(),
          type: 'text',
        }}
        name='search'
        onChange={local.onSearch}
      />
      <div class='flex justify-between gap-2'>
        <Select
          itemComponent={(props) => (
            <SelectItem item={props.item}>{props.item.textValue}</SelectItem>
          )}
          onChange={(value) => {
            if (value) {
              local.onPageSizeChange(value);
            }
          }}
          options={PER_PAGE_OPTIONS}
          value={local.pageSize}
        >
          <SelectTrigger class='gap-1.5 whitespace-nowrap px-2'>
            <SelectValue<number>>
              {(state) => `${t('common.perPage')}: ${state.selectedOption()}`}
            </SelectValue>
          </SelectTrigger>
          <SelectContent />
        </Select>

        <Show
          fallback={<Skeleton class='bg-secondary' radius={6} width={200} />}
          when={!local.loading}
        >
          <Show when={local.pageCount > 1}>
            <CombinedPagination
              count={local.pageCount}
              disabled={local.loading}
              onPageChange={local.onPageChange}
              page={local.currentPage}
              siblingCount={1}
            />
          </Show>
        </Show>
      </div>
    </div>
  );
};
