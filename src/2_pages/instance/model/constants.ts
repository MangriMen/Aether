import type { DialogFilter } from '@tauri-apps/plugin-dialog';

import { ContentType } from '@/entities/instances';

const ALL_FILTER: DialogFilter = {
  name: 'All',
  extensions: ['*'],
};

export const OPEN_FILTERS_BY_CONTENT_TYPE: Record<
  Exclude<ContentType, typeof ContentType.Modpack>,
  DialogFilter[] | undefined
> = {
  [ContentType.Mod]: [
    {
      name: 'Mod',
      extensions: ['jar'],
    },
    ALL_FILTER,
  ],
  [ContentType.DataPack]: [
    {
      name: 'Data Pack',
      extensions: ['zip'],
    },
    ALL_FILTER,
  ],
  [ContentType.ResourcePack]: [
    {
      name: 'Resource Pack',
      extensions: ['zip'],
    },
    ALL_FILTER,
  ],

  [ContentType.ShaderPack]: [
    {
      name: 'Shader Pack',
      extensions: ['zip'],
    },
    ALL_FILTER,
  ],
};
