import type { Component } from 'solid-js';

import { createMemo, splitProps } from 'solid-js';

import type { PartialBy } from '@/shared/utils';

import type { PaginationRootProps } from '../primitives/Pagination';

import {
  Pagination,
  PaginationEllipsis,
  PaginationItem,
  PaginationItems,
  PaginationNext,
  PaginationPrevious,
} from '../primitives/Pagination';

export type CombinedPaginationProps = PartialBy<
  PaginationRootProps<'div'>,
  'itemComponent' | 'ellipsisComponent'
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
      itemComponent={itemComponent()}
      ellipsisComponent={ellipsisComponent()}
      {...others}
    >
      <PaginationPrevious />
      <PaginationItems />
      <PaginationNext />
    </Pagination>
  );
};
