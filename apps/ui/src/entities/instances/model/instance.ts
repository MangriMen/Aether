import type { InstanceDto } from '../api';

export type InstanceSettings = Pick<
  Instance,
  'launchArgs' | 'envVars' | 'memory' | 'window' | 'hooks'
>;

export type Instance = InstanceDto;
