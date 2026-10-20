import type { Accessor, JSX } from 'solid-js';

import type { TabConfig } from '@/shared/ui-kit';

type ImporterTabProps = {
  /** Rendered with the submitting state so the submit button can show it. */
  footerButtons: (isSubmitting: Accessor<boolean>) => JSX.Element;
  onSubmit?: () => void;
};

export type ImporterTabConfig = TabConfig<string, ImporterTabProps>;
