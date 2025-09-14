import type { DialogFilter } from '@tauri-apps/plugin-dialog';

import { ContentType } from '@/entities/instances';

const ALL_FILTER: DialogFilter = {
  extensions: ['*'],
  name: 'All',
};

export const OPEN_FILTERS_BY_CONTENT_TYPE: Record<
  ContentType,
  DialogFilter[] | undefined
> = {
  [ContentType.DataPack]: [
    {
      extensions: ['zip'],
      name: 'Data Pack',
    },
    ALL_FILTER,
  ],
  [ContentType.Mod]: [
    {
      extensions: ['jar'],
      name: 'Mod',
    },
    ALL_FILTER,
  ],
  [ContentType.ResourcePack]: [
    {
      extensions: ['zip'],
      name: 'Resource Pack',
    },
    ALL_FILTER,
  ],

  [ContentType.ShaderPack]: [
    {
      extensions: ['zip'],
      name: 'Shader Pack',
    },
    ALL_FILTER,
  ],
};
