import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import type { JSX, ValidComponent } from 'solid-js';

import * as PaginationPrimitive from '@kobalte/core/pagination';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import { buttonVariants } from './Button';

const PaginationItems = PaginationPrimitive.Items;

type PaginationRootProps<T extends ValidComponent = 'nav'> = {
  class?: string | undefined;
} & PaginationPrimitive.PaginationRootProps<T>;

const Pagination = <T extends ValidComponent = 'nav'>(
  props: PolymorphicProps<T, PaginationRootProps<T>>,
) => {
  const [local, others] = splitProps(props as PaginationRootProps, ['class']);
  return (
    <PaginationPrimitive.Root
      class={cn(
        '[&>*]:flex [&>*]:flex-row [&>*]:items-center [&>*]:gap-1',
        local.class,
      )}
      {...others}
    />
  );
};

type PaginationItemProps<T extends ValidComponent = 'button'> = {
  class?: string | undefined;
} & PaginationPrimitive.PaginationItemProps<T>;

const PaginationItem = <T extends ValidComponent = 'button'>(
  props: PolymorphicProps<T, PaginationItemProps<T>>,
) => {
  const [local, others] = splitProps(props as PaginationItemProps, ['class']);
  return (
    <PaginationPrimitive.Item
      class={cn(
        buttonVariants({
          variant: 'ghost',
        }),
        'size-10 p-0 data-[current]:border',
        local.class,
      )}
      {...others}
    />
  );
};

type PaginationEllipsisProps<T extends ValidComponent = 'div'> = {
  class?: string | undefined;
} & PaginationPrimitive.PaginationEllipsisProps<T>;

const PaginationEllipsis = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, PaginationEllipsisProps<T>>,
) => {
  const [local, others] = splitProps(props as PaginationEllipsisProps, [
    'class',
  ]);
  return (
    <PaginationPrimitive.Ellipsis
      class={cn('flex size-10 items-center justify-center', local.class)}
      {...others}
    >
      <svg
        class='size-4'
        fill='none'
        stroke='currentColor'
        stroke-linecap='round'
        stroke-linejoin='round'
        stroke-width='2'
        viewBox='0 0 24 24'
        xmlns='http://www.w3.org/2000/svg'
      >
        <circle cx='12' cy='12' r='1' />
        <circle cx='19' cy='12' r='1' />
        <circle cx='5' cy='12' r='1' />
      </svg>
      <span class='sr-only'>More pages</span>
    </PaginationPrimitive.Ellipsis>
  );
};

type PaginationPreviousProps<T extends ValidComponent = 'button'> = {
  children?: JSX.Element;
  class?: string | undefined;
} & PaginationPrimitive.PaginationPreviousProps<T>;

const PaginationPrevious = <T extends ValidComponent = 'button'>(
  props: PolymorphicProps<T, PaginationPreviousProps<T>>,
) => {
  const [local, others] = splitProps(props as PaginationPreviousProps, [
    'class',
    'children',
  ]);
  return (
    <PaginationPrimitive.Previous
      class={cn(
        buttonVariants({
          variant: 'ghost',
        }),
        'gap-1 p-0 size-10',
        local.class,
      )}
      {...others}
    >
      {local.children ?? (
        <>
          <svg
            class='size-4'
            fill='none'
            stroke='currentColor'
            stroke-linecap='round'
            stroke-linejoin='round'
            stroke-width='2'
            viewBox='0 0 24 24'
            xmlns='http://www.w3.org/2000/svg'
          >
            <path d='M15 6l-6 6l6 6' />
          </svg>
          {/* <span>Previous</span> */}
        </>
      )}
    </PaginationPrimitive.Previous>
  );
};

type PaginationNextProps<T extends ValidComponent = 'button'> = {
  children?: JSX.Element;
  class?: string | undefined;
} & PaginationPrimitive.PaginationNextProps<T>;

const PaginationNext = <T extends ValidComponent = 'button'>(
  props: PolymorphicProps<T, PaginationNextProps<T>>,
) => {
  const [local, others] = splitProps(props as PaginationNextProps, [
    'class',
    'children',
  ]);
  return (
    <PaginationPrimitive.Next
      class={cn(
        buttonVariants({
          variant: 'ghost',
        }),
        'gap-1 p-0 size-10',
        local.class,
      )}
      {...others}
    >
      {local.children ?? (
        <>
          {/* <span>Next</span> */}
          <svg
            class='size-4'
            fill='none'
            stroke='currentColor'
            stroke-linecap='round'
            stroke-linejoin='round'
            stroke-width='2'
            viewBox='0 0 24 24'
            xmlns='http://www.w3.org/2000/svg'
          >
            <path d='M9 6l6 6l-6 6' />
          </svg>
        </>
      )}
    </PaginationPrimitive.Next>
  );
};

export type { PaginationRootProps };

export {
  Pagination,
  PaginationEllipsis,
  PaginationItem,
  PaginationItems,
  PaginationNext,
  PaginationPrevious,
};
