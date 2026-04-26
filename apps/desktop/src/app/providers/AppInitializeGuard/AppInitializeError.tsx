import {
  createSignal,
  onCleanup,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { cn } from '@/shared/lib';

export type AppInitializeErrorProps = ComponentProps<'div'> & {
  error?: string;
};

export const AppInitializeError: Component<AppInitializeErrorProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['error', 'class']);

  const [errorHasBeenCopied, setErrorHasBeenCopied] = createSignal(false);

  let errorHasBeenCopiedTimer = 0;

  const handleCopyError = () => {
    if (!local.error || typeof navigator === 'undefined') {
      return;
    }

    try {
      navigator.clipboard.writeText(local.error);
    } finally {
      setErrorHasBeenCopied(true);
      errorHasBeenCopiedTimer = setTimeout(() => {
        setErrorHasBeenCopied(false);
      }, 2500);
    }
  };

  onCleanup(() => {
    clearTimeout(errorHasBeenCopiedTimer);
  });

  return (
    <div
      class={cn(
        'flex grow flex-col gap-8 items-center justify-center px-8 pb-8 overflow-hidden',
        local.class,
      )}
      {...others}
    >
      <div class='flex flex-col items-center justify-center gap-2'>
        <h1 class='text-4xl font-bold text-destructive'>
          Initialization Error
        </h1>

        <div class='flex flex-col items-center gap-0.5'>
          <p class='text-xl'>The app encountered a problem during startup.</p>
          <p class='text-lg text-muted-foreground'>
            Try restarting the app. If the issue persists, please <br />
            reinstall it or <b>send the log files to the developer</b>.
          </p>
        </div>
      </div>

      <Show when={local.error}>
        {(error) => (
          <div class='flex max-h-max min-w-96 max-w-screen-2xl grow flex-col gap-1 overflow-hidden rounded-lg border border-destructive/20 bg-destructive/15 p-4'>
            <button
              class={cn(
                'rounded bg-destructive px-3 py-1 text-xs font-medium text-destructive-foreground transition-colors hover:bg-destructive/solid-hover',
                {
                  'bg-success text-success-foreground hover:bg-success/solid-hover':
                    errorHasBeenCopied(),
                },
              )}
              onClick={handleCopyError}
            >
              <Show when={errorHasBeenCopied()} fallback='Copy error'>
                Copied!
              </Show>
            </button>
            <p class='overflow-y-auto break-all font-mono text-sm leading-relaxed text-destructive-foreground'>
              {error()}
            </p>
          </div>
        )}
      </Show>
    </div>
  );
};
