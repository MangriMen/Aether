import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import type { Component, ComponentProps, JSX, ValidComponent } from 'solid-js';

import * as DropdownMenuPrimitive from '@kobalte/core/dropdown-menu';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

const DropdownMenuTrigger = DropdownMenuPrimitive.Trigger;
const DropdownMenuPortal = DropdownMenuPrimitive.Portal;
const DropdownMenuSub = DropdownMenuPrimitive.Sub;
const DropdownMenuGroup = DropdownMenuPrimitive.Group;
const DropdownMenuRadioGroup = DropdownMenuPrimitive.RadioGroup;

const DropdownMenu: Component<DropdownMenuPrimitive.DropdownMenuRootProps> = (
  props,
) => {
  return <DropdownMenuPrimitive.Root gutter={4} {...props} />;
};

type DropdownMenuContentProps<T extends ValidComponent = 'div'> = {
  class?: string | undefined;
} & DropdownMenuPrimitive.DropdownMenuContentProps<T>;

const DropdownMenuContent = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, DropdownMenuContentProps<T>>,
) => {
  const [, rest] = splitProps(props as DropdownMenuContentProps, ['class']);
  return (
    <DropdownMenuPrimitive.Portal>
      <DropdownMenuPrimitive.Content
        class={cn(
          'z-50 min-w-32 origin-[var(--kb-menu-content-transform-origin)] animate-content-hide overflow-hidden rounded-md border bg-popover p-1 text-popover-foreground shadow-md data-[expanded]:animate-content-show',
          props.class,
        )}
        {...rest}
      />
    </DropdownMenuPrimitive.Portal>
  );
};

type DropdownMenuItemProps<T extends ValidComponent = 'div'> = {
  class?: string | undefined;
} & DropdownMenuPrimitive.DropdownMenuItemProps<T>;

const DropdownMenuItem = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, DropdownMenuItemProps<T>>,
) => {
  const [, rest] = splitProps(props as DropdownMenuItemProps, ['class']);
  return (
    <DropdownMenuPrimitive.Item
      class={cn(
        'relative flex cursor-default select-none items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-none transition-colors focus:bg-accent focus:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50',
        props.class,
      )}
      {...rest}
    />
  );
};

const DropdownMenuShortcut: Component<ComponentProps<'span'>> = (props) => {
  const [, rest] = splitProps(props, ['class']);
  return (
    <span
      class={cn('ml-auto text-xs tracking-widest opacity-60', props.class)}
      {...rest}
    />
  );
};

const DropdownMenuLabel: Component<
  { inset?: boolean } & ComponentProps<'div'>
> = (props) => {
  const [, rest] = splitProps(props, ['class', 'inset']);
  return (
    <div
      class={cn(
        'px-2 py-1.5 text-sm font-semibold',
        props.inset && 'pl-8',
        props.class,
      )}
      {...rest}
    />
  );
};

type DropdownMenuSeparatorProps<T extends ValidComponent = 'hr'> = {
  class?: string | undefined;
} & DropdownMenuPrimitive.DropdownMenuSeparatorProps<T>;

const DropdownMenuSeparator = <T extends ValidComponent = 'hr'>(
  props: PolymorphicProps<T, DropdownMenuSeparatorProps<T>>,
) => {
  const [, rest] = splitProps(props as DropdownMenuSeparatorProps, ['class']);
  return (
    <DropdownMenuPrimitive.Separator
      class={cn('-mx-1 my-1 h-px bg-muted', props.class)}
      {...rest}
    />
  );
};

type DropdownMenuSubTriggerProps<T extends ValidComponent = 'div'> = {
  children?: JSX.Element;
  class?: string | undefined;
} & DropdownMenuPrimitive.DropdownMenuSubTriggerProps<T>;

const DropdownMenuSubTrigger = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, DropdownMenuSubTriggerProps<T>>,
) => {
  const [, rest] = splitProps(props as DropdownMenuSubTriggerProps, [
    'class',
    'children',
  ]);
  return (
    <DropdownMenuPrimitive.SubTrigger
      class={cn(
        'flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none focus:bg-accent data-[state=open]:bg-accent',
        props.class,
      )}
      {...rest}
    >
      {props.children}
      <svg
        class='ml-auto size-4'
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
    </DropdownMenuPrimitive.SubTrigger>
  );
};

type DropdownMenuSubContentProps<T extends ValidComponent = 'div'> = {
  class?: string | undefined;
} & DropdownMenuPrimitive.DropdownMenuSubContentProps<T>;

const DropdownMenuSubContent = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, DropdownMenuSubContentProps<T>>,
) => {
  const [, rest] = splitProps(props as DropdownMenuSubContentProps, ['class']);
  return (
    <DropdownMenuPrimitive.SubContent
      class={cn(
        'z-50 min-w-32 origin-[var(--kb-menu-content-transform-origin)] overflow-hidden rounded-md border bg-popover p-1 text-popover-foreground shadow-md animate-in',
        props.class,
      )}
      {...rest}
    />
  );
};

type DropdownMenuCheckboxItemProps<T extends ValidComponent = 'div'> = {
  children?: JSX.Element;
  class?: string | undefined;
} & DropdownMenuPrimitive.DropdownMenuCheckboxItemProps<T>;

const DropdownMenuCheckboxItem = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, DropdownMenuCheckboxItemProps<T>>,
) => {
  const [, rest] = splitProps(props as DropdownMenuCheckboxItemProps, [
    'class',
    'children',
  ]);
  return (
    <DropdownMenuPrimitive.CheckboxItem
      class={cn(
        'relative flex cursor-default select-none items-center rounded-sm py-1.5 pl-8 pr-2 text-sm outline-none transition-colors focus:bg-accent focus:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50',
        props.class,
      )}
      {...rest}
    >
      <span class='absolute left-2 flex size-3.5 items-center justify-center'>
        <DropdownMenuPrimitive.ItemIndicator>
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
            <path d='M5 12l5 5l10 -10' />
          </svg>
        </DropdownMenuPrimitive.ItemIndicator>
      </span>
      {props.children}
    </DropdownMenuPrimitive.CheckboxItem>
  );
};

type DropdownMenuGroupLabelProps<T extends ValidComponent = 'span'> = {
  class?: string | undefined;
} & DropdownMenuPrimitive.DropdownMenuGroupLabelProps<T>;

const DropdownMenuGroupLabel = <T extends ValidComponent = 'span'>(
  props: PolymorphicProps<T, DropdownMenuGroupLabelProps<T>>,
) => {
  const [, rest] = splitProps(props as DropdownMenuGroupLabelProps, ['class']);
  return (
    <DropdownMenuPrimitive.GroupLabel
      class={cn('px-2 py-1.5 text-sm font-semibold', props.class)}
      {...rest}
    />
  );
};

type DropdownMenuRadioItemProps<T extends ValidComponent = 'div'> = {
  children?: JSX.Element;
  class?: string | undefined;
} & DropdownMenuPrimitive.DropdownMenuRadioItemProps<T>;

const DropdownMenuRadioItem = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, DropdownMenuRadioItemProps<T>>,
) => {
  const [, rest] = splitProps(props as DropdownMenuRadioItemProps, [
    'class',
    'children',
  ]);
  return (
    <DropdownMenuPrimitive.RadioItem
      class={cn(
        'relative flex cursor-default select-none items-center rounded-sm py-1.5 pl-8 pr-2 text-sm outline-none transition-colors focus:bg-accent focus:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50',
        props.class,
      )}
      {...rest}
    >
      <span class='absolute left-2 flex size-3.5 items-center justify-center'>
        <DropdownMenuPrimitive.ItemIndicator>
          <svg
            class='size-2 fill-current'
            fill='none'
            stroke='currentColor'
            stroke-linecap='round'
            stroke-linejoin='round'
            stroke-width='2'
            viewBox='0 0 24 24'
            xmlns='http://www.w3.org/2000/svg'
          >
            <path d='M12 12m-9 0a9 9 0 1 0 18 0a9 9 0 1 0 -18 0' />
          </svg>
        </DropdownMenuPrimitive.ItemIndicator>
      </span>
      {props.children}
    </DropdownMenuPrimitive.RadioItem>
  );
};

export {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuGroupLabel,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuPortal,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
};
