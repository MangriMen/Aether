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

  const currentParams = createMemo(() =>
    decodeContentSearchParams(searchParams),
  );

  const setParams = (params: ContentPageSearchParams) => {
    setSearchParams(encodeContentSearchParams(params));
  };

  return [currentParams, setParams];
};
