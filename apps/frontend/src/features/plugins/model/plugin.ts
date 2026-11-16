import type {
  HostFunctionRiskDto,
  PluginDto,
} from '@/platform/bindings/plugin';

export type Plugin = PluginDto;

/** What a plugin can do through a host function it declares that the user is warned about. */
export type HostFunctionRisk = HostFunctionRiskDto;
