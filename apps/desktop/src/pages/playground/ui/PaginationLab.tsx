import {
  Pagination,
  PaginationEllipsis,
  PaginationItem,
  PaginationItems,
  PaginationNext,
  PaginationPrevious,
} from '../../../shared/ui';
import { ComponentShelf } from './ComponentShelf';

export const PaginationLab = () => {
  return (
    <ComponentShelf title='Pagination'>
      <Pagination
        count={10}
        fixedItems
        itemComponent={(props) => (
          <PaginationItem page={props.page}>{props.page}</PaginationItem>
        )}
        ellipsisComponent={() => <PaginationEllipsis />}
      >
        <PaginationPrevious />
        <PaginationItems />
        <PaginationNext />
      </Pagination>
    </ComponentShelf>
  );
};
