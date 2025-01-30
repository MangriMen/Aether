import type { Component } from 'solid-js';

export interface Plugin {
  name: string;
  version: string;
  id: string;
  description: string;
  component: Component<PluginComponentProps>;
}

export interface PluginComponentProps {
  onSubmit: () => void;
}
