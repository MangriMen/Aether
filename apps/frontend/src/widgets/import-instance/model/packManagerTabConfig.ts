import type { Component, JSX } from 'solid-js';

/** Props that the `ImportInstanceView` passes to every tab component.
 *  Plugin-specific props (pluginId, capabilityId, etc.) are captured
 *  via closure in `packManagerToTab`. */
export type PackManagerTabProps = {
  footerButtons: JSX.Element;
  onSubmit?: () => void;
};

export type PackManagerTabConfig = {
  value: string;
  label: string;
  icon?: string;
  component: Component<PackManagerTabProps>;
};
