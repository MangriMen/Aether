import { closeToast, showToast } from '@/shared/ui-kit';

interface CommandNotAnsweredText {
  title: string | undefined;
  description: string | undefined;
}

/** The toast each plugin has up, so that several expired questions leave one message. */
const shown = new Map<string, number>();

/**
 * A plugin asked to run a program and nobody answered, so nothing ran. The user was away, so
 * the message stays until they close it: the failure the plugin reports is a generic one that
 * does not say why. The text names the plugin only; the command stays out of it.
 */
export const showCommandNotAnswered = (
  pluginId: string,
  text: CommandNotAnsweredText,
) => {
  if (shown.has(pluginId)) {
    return;
  }

  const id = showToast({
    variant: 'warning',
    title: text.title,
    description: text.description,
    persistent: true,
    onDismiss: () => shown.delete(pluginId),
  });
  shown.set(pluginId, id);
};

/** The plugin asks again, so the message about the last time is out of date. */
export const closeCommandNotAnswered = (pluginId: string) => {
  const id = shown.get(pluginId);

  if (id !== undefined) {
    shown.delete(pluginId);
    closeToast(id);
  }
};
