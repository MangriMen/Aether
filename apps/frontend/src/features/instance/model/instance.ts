import type { InstanceDto } from '@/platform/bindings/instance';

export type InstanceSettings = Pick<
  Instance,
  'launchArgs' | 'envVars' | 'memory' | 'window' | 'hooks'
>;

export interface Instance extends Omit<InstanceDto, 'timePlayed'> {
  timePlayed: number;
}
