import type { PolymorphicProps } from '@kobalte/core/polymorphic';
import type { VariantProps } from 'class-variance-authority';
import type { JSX, ValidComponent } from 'solid-js';

import * as ToastPrimitive from '@kobalte/core/toast';
import { cva } from 'class-variance-authority';
import { Match, splitProps, Switch } from 'solid-js';
import { Portal } from 'solid-js/web';

import { cn } from '@/shared/lib';

// eslint-disable-next-line tailwindcss/no-custom-classname
const toastVariants = cva(
  'group pointer-events-auto relative flex w-full items-center justify-between space-x-4 overflow-hidden rounded-md border p-6 pr-8 shadow-lg transition-all data-[swipe=cancel]:translate-x-0 data-[swipe=end]:translate-x-[var(--kb-toast-swipe-end-x)] data-[swipe=move]:translate-x-[var(--kb-toast-swipe-move-x)] data-[swipe=move]:transition-none data-[opened]:animate-in data-[closed]:animate-out data-[swipe=end]:animate-out data-[closed]:fade-out-80 data-[closed]:slide-out-to-right-full data-[opened]:slide-in-from-top-full data-[opened]:sm:slide-in-from-bottom-full',
  {
    defaultVariants: {
      variant: 'default',
    },
    variants: {
      variant: {
        default: 'border bg-background text-foreground',
        destructive:
          'destructive group border-destructive bg-destructive text-destructive-foreground',
        error: 'error border-error-foreground bg-error text-error-foreground',
        success:
          'success border-success-foreground bg-success text-success-foreground',
        warning:
          'warning border-warning-foreground bg-warning text-warning-foreground',
        // TODO: rework toast variants design
        warningFilled:
          'warning border-warning-foreground bg-warning-foreground text-warning',
      },
    },
  },
);
type ToastListProps<T extends ValidComponent = 'ol'> = {
  class?: string | undefined;
} & ToastPrimitive.ToastListProps<T>;

type ToastVariant = NonNullable<VariantProps<typeof toastVariants>['variant']>;

const Toaster = <T extends ValidComponent = 'ol'>(
  props: PolymorphicProps<T, ToastListProps<T>>,
) => {
  const [local, others] = splitProps(props as ToastListProps, ['class']);
  return (
    <Portal>
      <ToastPrimitive.Region>
        <ToastPrimitive.List
          class={cn(
            'fixed top-0 z-[100] flex max-h-screen w-full flex-col-reverse gap-2 p-4 sm:bottom-0 sm:right-0 sm:top-auto sm:flex-col md:max-w-[420px]',
            local.class,
          )}
          {...others}
        />
      </ToastPrimitive.Region>
    </Portal>
  );
};

type ToastRootProps<T extends ValidComponent = 'li'> = {
  class?: string | undefined;
} & ToastPrimitive.ToastRootProps<T> &
  VariantProps<typeof toastVariants>;

const Toast = <T extends ValidComponent = 'li'>(
  props: PolymorphicProps<T, ToastRootProps<T>>,
) => {
  const [local, others] = splitProps(props as ToastRootProps, [
    'class',
    'variant',
  ]);
  return (
    <ToastPrimitive.Root
      class={cn(toastVariants({ variant: local.variant }), local.class)}
      {...others}
    />
  );
};

type ToastCloseButtonProps<T extends ValidComponent = 'button'> = {
  class?: string | undefined;
} & ToastPrimitive.ToastCloseButtonProps<T>;

const ToastClose = <T extends ValidComponent = 'button'>(
  props: PolymorphicProps<T, ToastCloseButtonProps<T>>,
) => {
  const [local, others] = splitProps(props as ToastCloseButtonProps, ['class']);
  return (
    <ToastPrimitive.CloseButton
      class={cn(
        'absolute right-2 top-2 rounded-md p-1 text-foreground/50 opacity-0 transition-opacity focus:opacity-100 focus:outline-none focus:ring-2 group-hover:opacity-100 group-[.destructive]:text-destructive-foreground group-[.error]:text-error-foreground group-[.success]:text-success-foreground group-[.warning]:text-warning',
        local.class,
      )}
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
        <path d='M18 6l-12 12' />
        <path d='M6 6l12 12' />
      </svg>
    </ToastPrimitive.CloseButton>
  );
};

type ToastTitleProps<T extends ValidComponent = 'div'> = {
  class?: string | undefined;
} & ToastPrimitive.ToastTitleProps<T>;

const ToastTitle = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, ToastTitleProps<T>>,
) => {
  const [local, others] = splitProps(props as ToastTitleProps, ['class']);
  return (
    <ToastPrimitive.Title
      class={cn('text-sm font-semibold', local.class)}
      {...others}
    />
  );
};

type ToastDescriptionProps<T extends ValidComponent = 'div'> = {
  class?: string | undefined;
} & ToastPrimitive.ToastDescriptionProps<T>;

const ToastDescription = <T extends ValidComponent = 'div'>(
  props: PolymorphicProps<T, ToastDescriptionProps<T>>,
) => {
  const [local, others] = splitProps(props as ToastDescriptionProps, ['class']);
  return (
    <ToastPrimitive.Description
      class={cn('text-sm opacity-90 [word-break:break-word]', local.class)}
      {...others}
    />
  );
};

interface ShowToastParams {
  description?: JSX.Element;
  duration?: number;
  title?: JSX.Element;
  variant?: ToastVariant;
}

function closeToast(id: number) {
  return ToastPrimitive.toaster.dismiss(id);
}

function showToast(props: ShowToastParams) {
  return ToastPrimitive.toaster.show((data) => (
    <Toast
      duration={props.duration}
      toastId={data.toastId}
      variant={props.variant}
    >
      <div class='grid w-full gap-1'>
        {props.title && <ToastTitle>{props.title}</ToastTitle>}
        {props.description && (
          <ToastDescription>{props.description}</ToastDescription>
        )}
      </div>
      <ToastClose />
    </Toast>
  ));
}

function showToastPromise<T, U>(
  promise: (() => Promise<T>) | Promise<T>,
  options: {
    duration?: number;
    error?: (error: U) => JSX.Element;
    loading?: JSX.Element;
    success?: (data: T) => JSX.Element;
  },
) {
  const variant: { [key in ToastPrimitive.ToastPromiseState]: ToastVariant } = {
    fulfilled: 'success',
    pending: 'default',
    rejected: 'error',
  };
  return ToastPrimitive.toaster.promise<T, U>(promise, (props) => (
    <Toast
      duration={options.duration}
      toastId={props.toastId}
      variant={variant[props.state]}
    >
      <Switch>
        <Match when={props.state === 'pending'}>{options.loading}</Match>
        <Match when={props.state === 'fulfilled'}>
          {options.success?.(props.data!)}
        </Match>
        <Match when={props.state === 'rejected'}>
          {options.error?.(props.error!)}
        </Match>
      </Switch>
    </Toast>
  ));
}

export type { ShowToastParams };

export {
  closeToast,
  showToast,
  showToastPromise,
  Toast,
  ToastClose,
  ToastDescription,
  Toaster,
  ToastTitle,
};
