import { convertFileSrc } from '@tauri-apps/api/core';

import type { InstanceDto } from '../../api';
import type { Instance } from '../instance';

export const instanceDtoToInstance = (instanceDto: InstanceDto): Instance => ({
  ...instanceDto,
  iconPath: instanceDto.iconPath
    ? // TODO: do something with assets path
      convertFileSrc(`ASSETS_PATH/${instanceDto.iconPath}`)
    : null,
});
