export type PluginsPaneState = 'loading' | 'error' | 'empty' | 'list';

export type PluginsPaneStateInput = {
  isError: boolean;
  isLoading: boolean;
  count: number | undefined;
};

/**
 * A failed first load must not read as "no plugins", and a failed refetch keeps the list it
 * already has.
 */
export const pluginsPaneState = ({
  isError,
  isLoading,
  count,
}: PluginsPaneStateInput): PluginsPaneState => {
  if (count === undefined) {
    if (isError) {
      return 'error';
    }

    return isLoading ? 'loading' : 'list';
  }

  return count === 0 ? 'empty' : 'list';
};
