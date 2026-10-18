import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import { fn } from 'storybook/test';

import { createArgSignal } from '../model/storyArgs';
import {
  Pagination,
  PaginationEllipsis,
  PaginationItem,
  PaginationItems,
  PaginationNext,
  PaginationPrevious,
} from './Pagination';

const meta: Meta = {
  title: 'ui-kit/Pagination',
  component: Pagination,
  tags: ['autodocs'],
  args: {
    count: 10,
    page: 1,
    siblingCount: 1,
    boundaryCount: 1,
    fixedItems: true,
    disabled: false,
    onPageChange: fn(),
  },
  argTypes: {
    count: { control: { type: 'number', min: 1, max: 100 } },
    page: { control: { type: 'number', min: 1, max: 100 } },
    siblingCount: { control: { type: 'number', min: 0, max: 5 } },
    boundaryCount: { control: { type: 'number', min: 0, max: 5 } },
    fixedItems: { control: 'boolean' },
    disabled: { control: 'boolean' },
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: (args) => {
    const [page, setPage] = createArgSignal(() => Number(args['page'] ?? 1), 1);

    return (
      <Pagination
        {...args}
        count={args['count']}
        page={page()}
        onPageChange={(value) => {
          setPage(value);
          args['onPageChange']?.(value);
        }}
        itemComponent={(props) => (
          <PaginationItem page={props.page}>{props.page}</PaginationItem>
        )}
        ellipsisComponent={() => <PaginationEllipsis />}
      >
        <PaginationPrevious />
        <PaginationItems />
        <PaginationNext />
      </Pagination>
    );
  },
};
