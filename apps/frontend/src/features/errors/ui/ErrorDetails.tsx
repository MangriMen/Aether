import { createSignal, onCleanup, type Component } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import {
  Button,
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '@/shared/ui-kit';

export interface ErrorDetailsProps {
  details: string;
  defaultOpen?: boolean;
}

/** The raw text of an error nobody could categorize: selectable, copyable, folded away by default. */
export const ErrorDetails: Component<ErrorDetailsProps> = (props) => {
  const [{ t }] = useTranslation();
  const [copied, setCopied] = createSignal(false);

  let timer: ReturnType<typeof setTimeout> | undefined;

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(props.details);
      setCopied(true);
      clearTimeout(timer);
      timer = setTimeout(() => setCopied(false), 2000);
    } catch {
      // The clipboard can be denied; the text stays selectable by hand.
    }
  };

  onCleanup(() => clearTimeout(timer));

  return (
    <Collapsible defaultOpen={props.defaultOpen} class='gap-1 flex flex-col'>
      <CollapsibleTrigger class='text-xs w-fit underline'>
        {t('errors.details')}
      </CollapsibleTrigger>
      <CollapsibleContent class='gap-1 flex flex-col'>
        <pre
          class='
            max-h-18 rounded-sm bg-black/20 p-2 font-mono text-xs/relaxed
            overflow-auto break-all whitespace-pre-wrap select-text
          '
        >
          {props.details}
        </pre>
        <Button
          size='sm'
          variant='secondary'
          class='w-fit'
          onClick={() => void handleCopy()}
        >
          {copied() ? t('errors.copied') : t('errors.copy')}
        </Button>
      </CollapsibleContent>
    </Collapsible>
  );
};
