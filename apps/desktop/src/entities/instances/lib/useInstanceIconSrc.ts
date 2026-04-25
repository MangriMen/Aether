import type { Accessor } from 'solid-js';

import {
  instanceToContentGetParams,
  useContent,
  type Instance,
} from '../model';

export const useInstanceIconSrc = (
  instance: Accessor<Instance>,
): Accessor<string | undefined> => {
  const content = useContent(() => instanceToContentGetParams(instance()));

  const iconSrc = () =>
    content.data?.iconUrl ?? instance().iconPath ?? undefined;

  return iconSrc;
};
