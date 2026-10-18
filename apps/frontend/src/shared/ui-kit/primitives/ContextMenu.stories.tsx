import type { Meta, StoryObj } from 'storybook-solidjs-vite';

import { createSignal } from 'solid-js';
import { fn } from 'storybook/test';

import {
  ContextMenu,
  ContextMenuCheckboxItem,
  ContextMenuContent,
  ContextMenuGroup,
  ContextMenuGroupLabel,
  ContextMenuItem,
  ContextMenuPortal,
  ContextMenuRadioGroup,
  ContextMenuRadioItem,
  ContextMenuSeparator,
  ContextMenuShortcut,
  ContextMenuSub,
  ContextMenuSubContent,
  ContextMenuSubTrigger,
  ContextMenuTrigger,
} from './ContextMenu';

const meta: Meta = {
  title: 'ui-kit/ContextMenu',
  component: ContextMenu,
  tags: ['autodocs'],
  args: { modal: true, preventScroll: true, onOpenChange: fn() },
  argTypes: {
    modal: { control: 'boolean' },
    preventScroll: { control: 'boolean' },
  },
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: (args) => {
    const [showGitLog, setShowGitLog] = createSignal(true);
    const [showHistory, setShowHistory] = createSignal(false);
    const [branch, setBranch] = createSignal('main');

    return (
      <ContextMenu {...args}>
        <ContextMenuTrigger class='h-37.5 w-75 rounded-md text-sm flex items-center justify-center border border-dashed'>
          Right click here.
        </ContextMenuTrigger>
        <ContextMenuPortal>
          <ContextMenuContent class='w-48'>
            <ContextMenuItem>
              <span>Commit</span>
              <ContextMenuShortcut>⌘+K</ContextMenuShortcut>
            </ContextMenuItem>
            <ContextMenuItem>
              <span>Push</span>
              <ContextMenuShortcut>⇧+⌘+K</ContextMenuShortcut>
            </ContextMenuItem>
            <ContextMenuItem disabled>
              <span>Update Project</span>
              <ContextMenuShortcut>⌘+T</ContextMenuShortcut>
            </ContextMenuItem>
            <ContextMenuSub overlap>
              <ContextMenuSubTrigger>GitHub</ContextMenuSubTrigger>
              <ContextMenuPortal>
                <ContextMenuSubContent>
                  <ContextMenuItem>Create Pull Request…</ContextMenuItem>
                  <ContextMenuItem>View Pull Requests</ContextMenuItem>
                  <ContextMenuItem>Sync Fork</ContextMenuItem>
                  <ContextMenuSeparator />
                  <ContextMenuItem>Open on GitHub</ContextMenuItem>
                </ContextMenuSubContent>
              </ContextMenuPortal>
            </ContextMenuSub>
            <ContextMenuSeparator />
            <ContextMenuCheckboxItem
              checked={showGitLog()}
              onChange={setShowGitLog}
            >
              Show Git Log
            </ContextMenuCheckboxItem>
            <ContextMenuCheckboxItem
              checked={showHistory()}
              onChange={setShowHistory}
            >
              Show History
            </ContextMenuCheckboxItem>
            <ContextMenuSeparator />
            <ContextMenuGroup>
              <ContextMenuGroupLabel>Branches</ContextMenuGroupLabel>
              <ContextMenuRadioGroup value={branch()} onChange={setBranch}>
                <ContextMenuRadioItem value='main'>main</ContextMenuRadioItem>
                <ContextMenuRadioItem value='develop'>
                  develop
                </ContextMenuRadioItem>
              </ContextMenuRadioGroup>
            </ContextMenuGroup>
          </ContextMenuContent>
        </ContextMenuPortal>
      </ContextMenu>
    );
  },
};
