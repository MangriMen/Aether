import { type Component, type ComponentProps, Show } from 'solid-js';

export type AppInitializeErrorProps = {
  message?: string;
} & ComponentProps<'div'>;

export const AppInitializeError: Component<AppInitializeErrorProps> = (
  props,
) => {
  return (
    <div class='flex h-full flex-col items-center justify-center' {...props}>
      <h1 class='text-4xl font-bold'>Error</h1>
      <p class='text-2xl'>The app is not initialized correctly.</p>
      <p class='text-2xl'>Please, try to reinstall the app.</p>
      <Show when={props.message}>
        {(message) => (
          <p class='text-2xl font-medium text-destructive'>
            Error: {message()}
          </p>
        )}
      </Show>
    </div>
  );
};
