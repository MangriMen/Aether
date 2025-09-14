import type { Component } from 'solid-js';

import { createMemo, splitProps } from 'solid-js';

import type { PartialBy } from '@/shared/model';

import type { PaginationRootProps } from './Pagination';

import {
  Pagination,
  PaginationEllipsis,
  PaginationItem,
  PaginationItems,
  PaginationNext,
  PaginationPrevious,
} from './Pagination';

export type CombinedPaginationProps = PartialBy<
  PaginationRootProps<'div'>,
  'ellipsisComponent' | 'itemComponent'
>;

export const CombinedPagination: Component<CombinedPaginationProps> = (
  props,
) => {
  const [local, others] = splitProps(props, [
    'itemComponent',
    'ellipsisComponent',
  ]);

  const itemComponent = createMemo(
    () =>
      local.itemComponent ??
      ((props: { page: number }) => (
        <PaginationItem page={props.page}>{props.page}</PaginationItem>
      )),
  );

  const ellipsisComponent = createMemo(
    () => local.ellipsisComponent ?? (() => <PaginationEllipsis />),
  );

  return (
    <Pagination
      ellipsisComponent={ellipsisComponent()}
      itemComponent={itemComponent()}
      {...others}
    >
      <PaginationPrevious />
      <PaginationItems />
      <PaginationNext />
    </Pagination>
  );
};
