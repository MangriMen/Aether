import {
  Component,
  createSignal,
  Match,
  Show,
  splitProps,
  Switch,
} from 'solid-js';
import { createStore } from 'solid-js/store';

import { cn } from '@/shared/lib';
import {
  Button,
  Field,
  TextField,
  TextFieldInput,
  TextFieldLabel,
} from '@/shared/ui';

import { SelectGameVersion } from '@/features/select-game-version';
import { SelectLoaderChips } from '@/features/select-loader-chips';
import { SelectLoaderVersion } from '@/features/select-loader-version';

import { CreateCustomInstanceFormProps, CreateCustomInstanceProps } from '.';

const loaders = [
  { name: 'Vanilla', value: 'vanilla' },
  { name: 'Forge', value: 'forge' },
  { name: 'Fabric', value: 'fabric' },
  { name: 'Quilt', value: 'quilt' },
];

const gameVersions = ['1.19.2', '1.19'];

export const CreateCustomInstance: Component<CreateCustomInstanceProps> = (
  props,
) => {
  const [isAdvanced, setIsAdvanced] = createSignal(false);

  const [local, others] = splitProps(props, ['class', 'footer']);

  const [fields, setFields] = createStore<CreateCustomInstanceFormProps>({
    gameVersion: undefined,
    loader: 'vanilla',
    loaderVersion: undefined,
  });

  const handleSubmit = (e: SubmitEvent) => {
    e.preventDefault();

    const data = new FormData(e.target as HTMLFormElement);

    const dataObject = { ...Object.fromEntries(data.entries()), ...fields };

    console.log(dataObject);
  };

  return (
    <form
      class={cn('flex flex-col gap-4', local.class)}
      onSubmit={handleSubmit}
      {...others}
    >
      <TextField class='flex flex-col gap-2'>
        <TextFieldLabel for='name'>Name</TextFieldLabel>
        <TextFieldInput
          type='text'
          id='name'
          name='name'
          class='max-w-[36ch]'
          maxLength={32}
          required
        />
      </TextField>

      <Field label='Game version'>
        <SelectGameVersion
          class='max-w-[31.5ch]'
          advanced={isAdvanced()}
          value={fields.gameVersion}
          defaultValue={gameVersions[0]}
          options={gameVersions}
          onChange={(value) => {
            if (value) {
              setFields('gameVersion', value);
            }
          }}
        />
      </Field>

      <Field label='Loader'>
        <SelectLoaderChips
          loaders={loaders}
          defaultValue={loaders[0].value}
          value={fields.loader}
          onChange={(value) => {
            if (value) {
              setFields('loader', value);
            }
          }}
        />
      </Field>

      <Show when={isAdvanced()}>
        <SelectLoaderVersion
          onChange={(value) => setFields('loaderVersion', value)}
        />
      </Show>

      <div class='flex w-full justify-between'>
        <Button size='sm' onClick={() => setIsAdvanced(!isAdvanced())}>
          <Switch>
            <Match when={isAdvanced()}>Hide advanced</Match>
            <Match when={!isAdvanced()}>Show advanced</Match>
          </Switch>
        </Button>

        {local.footer}
      </div>
    </form>
  );
};
