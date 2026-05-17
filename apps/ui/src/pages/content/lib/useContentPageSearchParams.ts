import type { Accessor } from 'solid-js';

import { useSearchParams } from '@solidjs/router';
import { createMemo } from 'solid-js';

import type { ContentPageSearchParams } from '../model';

import {
  decodeContentSearchParams,
  encodeContentSearchParams,
} from './contentSearchParams';

export const useContentPageSearchParams = (): [
  Accessor<ContentPageSearchParams>,
  (value: ContentPageSearchParams) => void,
] => {
  const [searchParams, setSearchParams] = useSearchParams();

  const params = createMemo(() => decodeContentSearchParams(searchParams));

  const isInitialSetup = createMemo(
    () => !params().providerId?.pluginId || !params().providerId?.capabilityId,
  );

  const setParams = (params: ContentPageSearchParams) => {
    setSearchParams(encodeContentSearchParams(params), {
      replace: isInitialSetup(),
    });
  };

  return [params, setParams];
};
