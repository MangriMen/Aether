import type { InstanceDto } from '../api';

export type InstanceSettings = Pick<
  Instance,
  'launchArgs' | 'envVars' | 'memory' | 'gameResolution' | 'hooks'
>;

export type Instance = InstanceDto;
