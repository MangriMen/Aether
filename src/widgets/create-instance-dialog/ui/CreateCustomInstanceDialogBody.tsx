import type { FieldValues, SubmitHandler } from '@modular-forms/solid';
import { createForm, getValue, setValue, zodForm } from '@modular-forms/solid';
import { createAsync } from '@solidjs/router';
import type { Component } from 'solid-js';
import {
  createMemo,
  createSignal,
  Match,
  Show,
  splitProps,
  Switch,
} from 'solid-js';

import { cn } from '@/shared/lib';
import {
  Button,
  Collapsible,
  CollapsibleContent,
  CombinedTextField,
  DialogFooter,
  LabeledField,
  showToast,
} from '@/shared/ui';

import type { InstanceCreateDto } from '@/entities/instances';
import {
  IncludeSnapshotsCheckbox,
  refetchInstances,
  createInstance,
} from '@/entities/instances';
import type { LoaderVersion, Version } from '@/entities/minecrafts';
import {
  getLoaderVersionsManifest,
  getMinecraftVersionManifest,
  ModLoader,
} from '@/entities/minecrafts';

import { useTranslate } from '@/shared/model';

import {
  CreateCustomInstanceSchema,
  filterGameVersions,
  filterGameVersionsForLoader,
  getLoaderVersionsForGameVersion,
  loaders,
  loaderVersionTypes,
} from '../model';

import { loaderManifestToMapped } from '../lib';
import { SelectGameVersion } from './SelectGameVersion';
import { LoaderVersionTypeChipsToggleGroup } from './LoaderVersionTypeChipsToggleGroup';
import { LoaderChipsToggleGroup } from './LoaderChipsToggleGroup';
import { SelectSpecificLoaderVersion } from './SelectSpecificLoaderVersion';
import type { DialogRootProps } from '@kobalte/core/dialog';
import type { z } from 'zod';

export type CreateCustomInstanceDialogBodyProps = { class?: string } & Pick<
  DialogRootProps,
  'onOpenChange'
>;

export type CreateCustomInstanceFormValues = FieldValues &
  z.infer<typeof CreateCustomInstanceSchema>;

export const CreateCustomInstanceDialogBody: Component<
  CreateCustomInstanceDialogBodyProps
> = (props) => {
  const [local, others] = splitProps(props, ['class', 'onOpenChange']);

  const [{ t }] = useTranslate();

  const [form, { Form, Field }] = createForm<CreateCustomInstanceFormValues>({
    validate: zodForm(CreateCustomInstanceSchema),
    initialValues: {
      loader: ModLoader.Vanilla,
      loaderVersionType: loaderVersionTypes[0].value,
    },
  });

  const [isCreating, setIsCreating] = createSignal(false);

  const [isAdvanced, setIsAdvanced] = createSignal(false);

  const [shouldIncludeSnapshots, setShouldIncludeSnapshots] =
    createSignal(false);

  const versionManifest = createAsync(() => getMinecraftVersionManifest());

  const forgeVersions = createAsync(async () =>
    loaderManifestToMapped(await getLoaderVersionsManifest(ModLoader.Forge)),
  );
  const fabricVersions = createAsync(async () =>
    loaderManifestToMapped(await getLoaderVersionsManifest(ModLoader.Fabric)),
  );
  const quiltVersions = createAsync(async () =>
    loaderManifestToMapped(await getLoaderVersionsManifest(ModLoader.Quilt)),
  );

  const loaderVersions = createMemo(() =>
    getLoaderVersionsForGameVersion(
      getValue(form, 'loader'),
      getValue(form, 'gameVersion'),
      {
        fabric: fabricVersions(),
        forge: forgeVersions(),
        quilt: quiltVersions(),
      },
    ),
  );

  const loaderGameVersions = createMemo(() =>
    filterGameVersionsForLoader(
      getValue(form, 'loader'),
      versionManifest()?.versions ?? [],
      {
        fabric: fabricVersions(),
        forge: forgeVersions(),
        quilt: quiltVersions(),
      },
    ),
  );

  const filteredGameVersions = createMemo(() =>
    filterGameVersions(loaderGameVersions(), shouldIncludeSnapshots()),
  );

  const handleSubmit: SubmitHandler<CreateCustomInstanceFormValues> = async (
    values,
  ) => {
    setIsCreating(true);

    const payload: InstanceCreateDto = {
      name: values.name,
      gameVersion: values.gameVersion,
      modLoader: values.loader,
      loaderVersion: values.loaderVersion,
    };

    try {
      props.onOpenChange?.(false);
      refetchInstances();

      await createInstance(payload);
    } catch (e) {
      console.error(e);
      showToast({
        title: `Failed to create instance ${payload.name}`,
        variant: 'destructive',
      });
    }

    setIsCreating(false);
  };

  return (
    <Form
      class={cn('flex flex-col gap-4', local.class)}
      onSubmit={handleSubmit}
      shouldActive
      {...others}
    >
      <Field name='name'>
        {(field, props) => (
          <CombinedTextField
            label={t('common.name')}
            value={field.value}
            errorMessage={field.error}
            inputProps={{
              class: 'max-w-[36ch]',
              maxLength: 32,
              autocomplete: 'off',
              type: 'text',
              ...props,
            }}
          />
        )}
      </Field>

      <LabeledField label={t('common.loader')}>
        <Field name='loader'>
          {(field) => (
            <LoaderChipsToggleGroup
              loaders={loaders}
              value={field.value}
              onChange={(value) => {
                setValue(form, 'loader', value as ModLoader);
                setValue(form, 'loaderVersion', undefined);
              }}
            />
          )}
        </Field>
      </LabeledField>

      <LabeledField label={t('common.gameVersion')}>
        <div class='flex items-start gap-2'>
          <Field name='gameVersion'>
            {(field, props) => (
              <SelectGameVersion
                class='max-w-[31.5ch]'
                placeholder={t('createInstance.gameVersionPlaceholder')}
                options={filteredGameVersions()}
                errorMessage={field.error}
                {...props}
                onChange={(value: Version | null) => {
                  if (value) {
                    setValue(form, 'gameVersion', value.id);
                  }
                }}
              />
            )}
          </Field>
          <IncludeSnapshotsCheckbox
            class='mt-2'
            show={isAdvanced()}
            checked={shouldIncludeSnapshots()}
            onChange={setShouldIncludeSnapshots}
          />
        </div>
      </LabeledField>

      <Collapsible
        open={isAdvanced() && getValue(form, 'loader') !== ModLoader.Vanilla}
      >
        <CollapsibleContent class='p-2'>
          <LabeledField label={t('createInstance.loaderVersion')}>
            <Field name='loaderVersionType'>
              {(field) => (
                <LoaderVersionTypeChipsToggleGroup
                  loaderTypes={loaderVersionTypes}
                  value={field.value}
                  onChange={(value) => {
                    if (value) {
                      setValue(form, 'loaderVersionType', value);
                    }
                  }}
                />
              )}
            </Field>
          </LabeledField>

          <Show when={getValue(form, 'loaderVersionType') === 'other'}>
            <Show
              when={getValue(form, 'gameVersion') !== undefined}
              fallback={
                <span class='italic'>
                  {t('createInstance.loaderVersionNoGameVersion')}
                </span>
              }
            >
              <LabeledField label={t('createInstance.loaderVersion')}>
                <Field name='loaderVersion'>
                  {(field, props) => (
                    <SelectSpecificLoaderVersion
                      placeholder={t('createInstance.loaderVersionPlaceholder')}
                      options={loaderVersions()}
                      errorMessage={field.error}
                      {...props}
                      onChange={(value: LoaderVersion | null) => {
                        if (value) {
                          setValue(form, 'loaderVersion', value.id);
                        }
                      }}
                    />
                  )}
                </Field>
              </LabeledField>
            </Show>
          </Show>
        </CollapsibleContent>
      </Collapsible>

      <DialogFooter>
        <Button
          class='mb-2 sm:mb-0 sm:mr-auto'
          variant='secondary'
          onClick={() => setIsAdvanced(!isAdvanced())}
        >
          <Switch>
            <Match when={isAdvanced()}>
              {t('createInstance.hideAdvanced')}
            </Match>
            <Match when={!isAdvanced()}>
              {t('createInstance.showAdvanced')}
            </Match>
          </Switch>
        </Button>

        <Button type='submit' disabled={isCreating()}>
          {t('common.create')}
        </Button>

        <Button variant='secondary' onClick={() => props.onOpenChange?.(false)}>
          {t('common.cancel')}
        </Button>
      </DialogFooter>
    </Form>
  );
};
