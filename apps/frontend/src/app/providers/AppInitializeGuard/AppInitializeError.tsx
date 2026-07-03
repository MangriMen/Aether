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
        `
          gap-8 px-8 pb-8 flex grow flex-col items-center justify-center
          overflow-hidden
        `,
        local.class,
      )}
      {...others}
    >
      <div class='gap-2 flex flex-col items-center justify-center'>
        <h1 class='text-4xl font-bold text-destructive'>
          Initialization Error
        </h1>

        <div class='gap-0.5 flex flex-col items-center'>
          <p class='text-xl'>The app encountered a problem during startup.</p>
          <p class='text-lg text-muted-foreground'>
            Try restarting the app. If the issue persists, please <br />
            reinstall it or <b>send the log files to the developer</b>.
          </p>
        </div>
      </div>

      <Show when={local.error}>
        {(error) => (
          <div
            class='
              min-w-96 gap-1 rounded-md border-destructive/20 bg-destructive/15
              p-4 flex max-h-max max-w-(--breakpoint-2xl) grow flex-col
              overflow-hidden border
            '
          >
            <button
              class={cn(
                `
                  rounded-sm bg-destructive px-3 py-1 text-xs font-medium
                  text-destructive-foreground
                  hover:bg-destructive/solid-hover
                  transition-colors
                `,
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
            <p
              class='
                font-mono text-sm/relaxed text-destructive-foreground
                overflow-y-auto break-all
              '
            >
              {error()}
            </p>
          </div>
        )}
      </Show>
    </div>
  );
};
