import type { CollectionNode } from '@kobalte/core';

import { createMemo, createSignal, type Component, type JSX } from 'solid-js';

import type { SettingsEntryProps } from '@/shared/settings-layout';

import {
  MAX_CUSTOM_FONT_FAMILY_LENGTH,
  sanitizeFontFamily,
  UI_FONTS,
  useThemeContext,
  type UiFont,
} from '@/features/appearance';
import { useSystemFonts } from '@/features/settings';
import { useTranslation } from '@/shared/i18n';
import { SettingsEntry } from '@/shared/settings-layout';
import {
  Combobox,
  ComboboxContent,
  ComboboxControl,
  ComboboxInput,
  ComboboxItem,
  ComboboxTrigger,
} from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

/** From this many rows the list draws only the ones in view. */
const VIRTUALIZE_FROM = 50;

type FontOption = { value: UiFont; label: string };

export type UiFontEntryProps = SettingsEntryProps;

export const UiFontEntry: Component<UiFontEntryProps> = (props) => {
  const [{ t }] = useTranslation();

  const [themeContext, { setUiFont, setCustomFontFamily }] = useThemeContext();

  const systemOption = createMemo<FontOption>(() => ({
    value: UI_FONTS.System,
    label: t('settings.uiFontSystem') ?? '',
  }));
  const interOption: FontOption = { value: UI_FONTS.Inter, label: 'Inter' };

  // What the user has typed so far, that is not applied before it is chosen.
  const [draft, setDraft] = createSignal('');

  // The typed name is the built-in choice of that name, or else a custom font of its own.
  const draftOption = createMemo<FontOption | undefined>(() => {
    const name = sanitizeFontFamily(draft());

    if (!name) {
      return undefined;
    }

    return (
      [systemOption(), interOption].find(
        (option) => option.label.toLowerCase() === name.toLowerCase(),
      ) ?? { value: UI_FONTS.Custom, label: name }
    );
  });

  const systemFonts = useSystemFonts();

  // The installed fonts, but not the ones the built-in choices already stand for. A name typed
  // in or stored that is not installed is shown in the field only, not as a row of the list.
  const installedOptions = createMemo<FontOption[]>(() => {
    const taken = new Set(
      [systemOption(), interOption].map((option) => option.label.toLowerCase()),
    );

    return (systemFonts.data ?? [])
      .filter((name) => !taken.has(name.toLowerCase()))
      .map((name) => ({ value: UI_FONTS.Custom, label: name }));
  });

  // The applied name that is not installed (not in the list), kept among the options only so
  // the field can show it: the list itself leaves it out.
  const unlistedOption = createMemo<FontOption | undefined>(() => {
    const stored = sanitizeFontFamily(themeContext.customFontFamily);
    const isListed = installedOptions().some(
      (option) => option.label.toLowerCase() === stored.toLowerCase(),
    );

    return themeContext.uiFont === UI_FONTS.Custom && stored && !isListed
      ? { value: UI_FONTS.Custom, label: stored }
      : undefined;
  });

  const options = createMemo(() => {
    const unlisted = unlistedOption();

    return [
      systemOption(),
      interOption,
      ...(unlisted ? [unlisted] : []),
      ...installedOptions(),
    ];
  });

  // Drawing only the rows in view costs more than it saves on a short list.
  const isVirtualized = () => options().length > VIRTUALIZE_FROM;

  // Quoted the way the interface font is, so the name is a safe `font-family` value.
  const previewStyle = (option: FontOption): JSX.CSSProperties | undefined => {
    if (option.value !== UI_FONTS.Custom) {
      return undefined;
    }

    const family = sanitizeFontFamily(option.label);

    return family ? { 'font-family': `'${family}', sans-serif` } : undefined;
  };

  // The row that closes the built-in choices, set off from the installed fonts after it.
  const hasSeparatorAfter = (option: FontOption) =>
    option.value === UI_FONTS.Inter && installedOptions().length > 0;

  const currentOption = createMemo<FontOption>(() => {
    if (themeContext.uiFont === UI_FONTS.Inter) {
      return interOption;
    }

    const stored = sanitizeFontFamily(themeContext.customFontFamily);

    return themeContext.uiFont === UI_FONTS.Custom && stored
      ? { value: UI_FONTS.Custom, label: stored }
      : systemOption();
  });

  const FontItem = (itemProps: {
    item: CollectionNode<FontOption>;
    style?: JSX.CSSProperties;
  }) => (
    <ComboboxItem
      class={cn(
        'whitespace-nowrap',
        hasSeparatorAfter(itemProps.item.rawValue) &&
          'after:absolute after:inset-x-2 after:bottom-0 after:h-px after:bg-border',
      )}
      item={itemProps.item}
      style={{
        ...itemProps.style,
        ...previewStyle(itemProps.item.rawValue),
      }}
    >
      {itemProps.item.rawValue.label}
    </ComboboxItem>
  );

  const [isOpen, setIsOpen] = createSignal(false);

  // Whether the highlighted option was picked with the arrow keys since the last typing.
  let isArrowPicking = false;

  const handleChange = (option: FontOption | null) => {
    if (!option) {
      return;
    }

    if (option.value === UI_FONTS.Custom) {
      setCustomFontFamily(option.label);
    }

    setUiFont(option.value);
    setDraft('');
    isArrowPicking = false;
  };

  // A name typed in is applied by Enter (when no listed option is picked instead) and by
  // leaving the field, not by every letter: the whole interface would flicker through the
  // half-typed names.
  const commitDraft = () => {
    const typed = draftOption();

    if (typed) {
      handleChange(typed);
    }

    setIsOpen(false);
  };

  return (
    <SettingsEntry
      title={t('settings.uiFont')}
      description={t('settings.uiFontDescription')}
      {...props}
    >
      <Combobox<FontOption>
        class='w-52'
        virtualized={isVirtualized()}
        triggerMode='focus'
        // Opened without typing, the list shows every choice, not only the one named in the field.
        defaultFilter={(option, inputValue) =>
          option.label !== unlistedOption()?.label &&
          (draft() === '' ||
            option.label.toLowerCase().includes(inputValue.toLowerCase()))
        }
        options={options()}
        optionValue='label'
        optionTextValue='label'
        optionLabel='label'
        value={currentOption()}
        onChange={handleChange}
        open={isOpen()}
        onOpenChange={setIsOpen}
        itemComponent={(itemProps) => <FontItem item={itemProps.item} />}
      >
        <ComboboxControl<FontOption>>
          <ComboboxInput
            maxLength={MAX_CUSTOM_FONT_FAMILY_LENGTH}
            onKeyDown={(
              e: KeyboardEvent & { currentTarget: HTMLInputElement },
            ) => {
              const input = e.currentTarget;

              if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
                isArrowPicking = true;
              }

              if (e.key === 'Escape') {
                setDraft('');
              }

              // The option picked with the arrow keys, else the typed name: not the option the
              // list focused by itself (the applied one, on opening). The combobox picks that one
              // first, so this goes after it.
              if (e.key === 'Enter') {
                const activeId = input.getAttribute('aria-activedescendant');
                const activeLabel = activeId
                  ? document.getElementById(activeId)?.textContent?.trim()
                  : undefined;
                const target =
                  isArrowPicking && activeLabel
                    ? options().find((option) => option.label === activeLabel)
                    : draftOption();

                queueMicrotask(() => {
                  if (target) {
                    handleChange(target);
                  }

                  setIsOpen(false);
                });
              }
            }}
            // Not the combobox's own onInputChange: it also reports the label it puts back
            // into the field when the focus leaves, which is not what was typed.
            onInput={(e: InputEvent & { currentTarget: HTMLInputElement }) => {
              isArrowPicking = false;
              setDraft(e.currentTarget.value);
            }}
            onFocus={(e: FocusEvent & { currentTarget: HTMLInputElement }) =>
              e.currentTarget.select()
            }
            onBlur={commitDraft}
            placeholder={t('settings.uiFontCustomPlaceholder')}
          />
          <ComboboxTrigger />
        </ComboboxControl>
        {/* A click outside the open list is leaving the field */}
        <ComboboxContent<FontOption>
          virtualized={isVirtualized()}
          class='max-h-72'
          onInteractOutside={commitDraft}
          itemComponent={(itemProps) => (
            <FontItem item={itemProps.item} style={itemProps.style} />
          )}
        />
      </Combobox>
    </SettingsEntry>
  );
};
