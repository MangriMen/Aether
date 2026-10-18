import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import { createSignal } from 'solid-js';
import { fn } from 'storybook/test';

import {
  createArgSignal,
  gutterArgType,
  placementArgType,
} from '../model/storyArgs';
import { Button } from './Button';
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuGroupLabel,
  DropdownMenuItem,
  DropdownMenuPortal,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from './DropdownMenu';

const meta: Meta = {
  title: 'ui-kit/DropdownMenu',
  component: DropdownMenu,
  tags: ['autodocs'],
  args: {
    open: false,
    modal: true,
    placement: 'bottom-start',
    gutter: 8,
    onOpenChange: fn(),
  },
  argTypes: {
    open: { control: 'boolean' },
    modal: { control: 'boolean' },
    placement: placementArgType,
    gutter: gutterArgType,
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: (args) => {
    const [open, setOpen] = createArgSignal(() => Boolean(args['open']), false);

    const [showGitLog, setShowGitLog] = createSignal(false);
    const [showHistory, setShowHistory] = createSignal(false);
    const [branch, setBranch] = createSignal('develop');

    return (
      <div class='flex-col'>
        <DropdownMenu
          {...args}
          open={open()}
          onOpenChange={(value) => {
            setOpen(value);
            args['onOpenChange']?.(value);
          }}
        >
          <DropdownMenuTrigger as={Button<'button'>}>
            Git Settings
          </DropdownMenuTrigger>
          <DropdownMenuContent class='w-48'>
            <DropdownMenuItem>
              <span>Commit</span>
              <DropdownMenuShortcut>⌘+K</DropdownMenuShortcut>
            </DropdownMenuItem>
            <DropdownMenuItem>
              <span>Push</span>
              <DropdownMenuShortcut>⇧+⌘+K</DropdownMenuShortcut>
            </DropdownMenuItem>
            <DropdownMenuItem>
              <span>Update Project</span>
              <DropdownMenuShortcut>⌘+T</DropdownMenuShortcut>
            </DropdownMenuItem>
            <DropdownMenuSub overlap>
              <DropdownMenuSubTrigger>GitHub</DropdownMenuSubTrigger>
              <DropdownMenuPortal>
                <DropdownMenuSubContent>
                  <DropdownMenuItem>Create Pull Request…</DropdownMenuItem>
                  <DropdownMenuItem>View Pull Requests</DropdownMenuItem>
                  <DropdownMenuItem>Sync Fork</DropdownMenuItem>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem>Open on GitHub</DropdownMenuItem>
                </DropdownMenuSubContent>
              </DropdownMenuPortal>
            </DropdownMenuSub>
            <DropdownMenuSeparator />
            <DropdownMenuCheckboxItem
              checked={showGitLog()}
              onChange={setShowGitLog}
            >
              Show Git Log
            </DropdownMenuCheckboxItem>
            <DropdownMenuCheckboxItem
              checked={showHistory()}
              onChange={setShowHistory}
            >
              Show History
            </DropdownMenuCheckboxItem>
            <DropdownMenuSeparator />
            <DropdownMenuGroup>
              <DropdownMenuGroupLabel>Branches</DropdownMenuGroupLabel>
              <DropdownMenuRadioGroup value={branch()} onChange={setBranch}>
                <DropdownMenuRadioItem value='main'>main</DropdownMenuRadioItem>
                <DropdownMenuRadioItem value='develop'>
                  develop
                </DropdownMenuRadioItem>
              </DropdownMenuRadioGroup>
            </DropdownMenuGroup>
          </DropdownMenuContent>
        </DropdownMenu>
        <p class='pt-2 text-sm text-gray-500'>
          Show Git Log: {showGitLog() ? 'yes' : 'no'}
        </p>
        <p class='pt-2 text-sm text-gray-500'>
          Show History: {showHistory() ? 'yes' : 'no'}
        </p>
        <p class='pt-2 text-sm text-gray-500'>Selected Branch: {branch()}</p>
      </div>
    );
  },
};
