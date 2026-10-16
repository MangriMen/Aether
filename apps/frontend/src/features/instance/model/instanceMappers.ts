import type { InstanceDto } from '@/platform/bindings/instance';

import type { Instance } from './instance';

export const instanceDtoToInstance = (instanceDto: InstanceDto): Instance => ({
  ...instanceDto,
  timePlayed: Number.parseInt(instanceDto.timePlayed),
});
