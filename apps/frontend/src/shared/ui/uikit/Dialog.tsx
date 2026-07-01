import type { PointerDownOutsideEvent } from '@kobalte/core';
import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import type { VariantProps } from 'class-variance-authority';
import type { Component, ComponentProps, JSX, ValidComponent } from 'solid-js';

import * as DialogPrimitive from '@kobalte/core/dialog';
import { cva } from 'class-variance-authority';
import { Show, splitProps } from 'solid-js';

import { cn } from '../../lib';
import { useTranslation } from '../../model';
import { IconButton } from '../components';

const Dialog = DialogPrimitive.Root;
const DialogTrigger = DialogPrimitive.Trigger;

const DialogPortal: Component<DialogPrimitive.DialogPortalProps> = (props) => {
  const [, rest] = splitProps(props, ['children']);
  return (
    <DialogPrimitive.Portal {...rest}>
      <div
        class='
          inset-0
          sm:items-center
          fixed z-50 flex items-start justify-center
        '
      >
        {props.children}
      </div>
    </DialogPrimitive.Portal>
  );
};

const dialogOverlayVariants = cva(
  `
    inset-0 backdrop-blur-sm
    data-closed:animate-out data-closed:fade-out-0
    data-expanded:animate-in data-expanded:fade-in-0
    fixed z-50
  `,
  {
    variants: {
      variant: {
        default: 'bg-background/80',
        destructive: 'bg-destructive/35',
        warning: 'bg-warning/35',
        unstyled: 'backdrop-blur-none',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  },
);

type DialogOverlayProps<T extends ValidComponent = 'div'> =
  DialogPrimitive.DialogOverlayProps<T> &
    VariantProps<typeof dialogOverlayVariants> & { class?: string | undefined };

const DialogOverlay = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, DialogOverlayProps<T>>,
) => {
  const [local, rest] = splitProps(props as DialogOverlayProps, [
    'variant',
    'class',
  ]);
  return (
    <DialogPrimitive.Overlay
      class={cn(dialogOverlayVariants({ variant: local.variant }), local.class)}
      {...rest}
    />
  );
};

type DialogContentProps<T extends ValidComponent = 'div'> =
  DialogPrimitive.DialogContentProps<T> &
    Pick<DialogOverlayProps, 'variant'> & {
      actions?: JSX.Element;
      showActions?: boolean;
      class?: string | undefined;
      children?: JSX.Element;
    };

const DialogContent = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, DialogContentProps<T>>,
) => {
  const [local, rest] = splitProps(props as DialogContentProps, [
    'variant',
    'actions',
    'showActions',
    'class',
    'children',
  ]);

  const onPointerDownOutsideGuard = (e: PointerDownOutsideEvent) =>
    e.target &&
    e.target instanceof HTMLElement &&
    e.target.dataset['ignoreOutsideClick']
      ? e.preventDefault()
      : undefined;

  const [{ t }] = useTranslation();

  return (
    <DialogPortal>
      <DialogOverlay variant={local.variant} />
      <DialogPrimitive.Content
        onPointerDownOutside={onPointerDownOutsideGuard}
        class={cn(
          local.variant !== 'unstyled' &&
            `
              max-w-lg gap-2 bg-popover/hard p-6 shadow-lg
              data-closed:animate-out data-closed:fade-out-0
              data-closed:zoom-out-95
              data-expanded:animate-in data-expanded:fade-in-0
              data-expanded:zoom-in-95
              sm:rounded-lg
              fixed top-1/2 left-1/2 z-50 grid max-h-[calc(100vh-80px)] w-full
              -translate-1/2 overflow-y-auto border duration-200
            `,
          local.class,
        )}
        {...rest}
      >
        {local.children}
        <Show when={local.showActions ?? true}>
          <div class='top-5.5 right-6 absolute flex'>
            {local.actions}
            <DialogPrimitive.CloseButton
              as={IconButton}
              size='sm'
              variant='ghost'
              class='
                data-expanded:bg-accent data-expanded:text-muted-foreground
                opacity-70 transition-opacity
                hover:opacity-100
              '
              title={t('common.close')}
            >
              <svg
                xmlns='http://www.w3.org/2000/svg'
                viewBox='0 0 24 24'
                fill='none'
                stroke='currentColor'
                stroke-width='2'
                stroke-linecap='round'
                stroke-linejoin='round'
                class='size-4'
              >
                <path d='M18 6l-12 12' />
                <path d='M6 6l12 12' />
              </svg>
              <span class='sr-only'>{t('common.close')}</span>
            </DialogPrimitive.CloseButton>
          </div>
        </Show>
      </DialogPrimitive.Content>
    </DialogPortal>
  );
};

const DialogHeader: Component<ComponentProps<'div'>> = (props) => {
  const [, rest] = splitProps(props, ['class']);
  return (
    <div
      class={cn(
        `
          space-y-1.5
          sm:text-left
          flex flex-col text-center
        `,
        props.class,
      )}
      {...rest}
    />
  );
};

const DialogFooter: Component<ComponentProps<'div'>> = (props) => {
  const [, rest] = splitProps(props, ['class']);
  return (
    <div
      class={cn(
        `
          gap-2
          sm:flex-row sm:justify-end
          flex flex-col-reverse
        `,
        props.class,
      )}
      {...rest}
    />
  );
};

type DialogTitleProps<T extends ValidComponent = 'h2'> =
  DialogPrimitive.DialogTitleProps<T> & {
    class?: string | undefined;
  };

const DialogTitle = <T extends ValidComponent = 'h2'>(
  props: PolymorphicProps<T, DialogTitleProps<T>>,
) => {
  const [, rest] = splitProps(props as DialogTitleProps, ['class']);
  return (
    <DialogPrimitive.Title
      class={cn(
        'text-lg font-semibold tracking-tight leading-none',
        props.class,
      )}
      {...rest}
    />
  );
};

type DialogDescriptionProps<T extends ValidComponent = 'p'> =
  DialogPrimitive.DialogDescriptionProps<T> & {
    class?: string | undefined;
  };

const DialogDescription = <T extends ValidComponent = 'p'>(
  props: PolymorphicProps<T, DialogDescriptionProps<T>>,
) => {
  const [, rest] = splitProps(props as DialogDescriptionProps, ['class']);
  return (
    <DialogPrimitive.Description
      class={cn('text-sm text-muted-foreground', props.class)}
      {...rest}
    />
  );
};

export type {
  DialogOverlayProps,
  DialogContentProps,
  DialogTitleProps,
  DialogDescriptionProps,
};

export {
  Dialog,
  DialogTrigger,
  DialogContent,
  DialogHeader,
  DialogFooter,
  DialogTitle,
  DialogDescription,
};
