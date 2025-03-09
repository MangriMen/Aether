import { cn } from '@/shared/lib';
import {
  CombinedPagination,
  CombinedTextField,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui';
import {
  createMemo,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

export type ContentFiltersProps = ComponentProps<'div'> & {
  pageCount: number;
  pageSize: number;
  currentPage: number;
  onSearch: (query: string) => void;
  onPageChange: (page: number) => void;
  onPageSizeChange: (pageSize: number) => void;
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
    'class',
  ]);

  const searchPlaceholder = createMemo(() => 'Search contents');

  return (
    <div class={cn('flex flex-col gap-2', local.class)} {...others}>
      <CombinedTextField
        class='w-full'
        name='search'
        inputProps={{ type: 'text', placeholder: searchPlaceholder() }}
        onChange={local.onSearch}
      />
      <div class='flex justify-between'>
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
          <SelectTrigger>
            <SelectValue>
              {(state) => `View: ${state.selectedOption()}`}
            </SelectValue>
          </SelectTrigger>
          <SelectContent />
        </Select>

        <CombinedPagination
          siblingCount={1}
          count={local.pageCount}
          page={local.currentPage}
          onPageChange={local.onPageChange}
        />
      </div>
    </div>
  );
};
