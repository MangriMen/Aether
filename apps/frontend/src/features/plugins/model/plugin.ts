import type {
  HostFunctionRiskDto,
  PluginDto,
  PluginUpdateDiffDto,
  PluginUpdatePreviewDto,
} from '@/platform/bindings/plugin';

export type Plugin = PluginDto;

/** What a plugin can do through a host function it declares that the user is warned about. */
export type HostFunctionRisk = HostFunctionRiskDto;

/** A release of an installed plugin as shown before the update to it. */
export type PluginUpdatePreview = PluginUpdatePreviewDto;

/** What a release asks for against the installed version, as the core compared it. */
export type PluginUpdateDiff = PluginUpdateDiffDto;
