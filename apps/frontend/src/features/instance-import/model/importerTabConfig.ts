import type { JSX } from 'solid-js';

import type { TabConfig } from '@/shared/ui-kit';

export type ImporterTabProps = {
  footerButtons: JSX.Element;
  onSubmit?: () => void;
};

export type ImporterTabConfig = TabConfig<string, ImporterTabProps>;
