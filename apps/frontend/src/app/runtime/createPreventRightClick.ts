export const createPreventRightClick =
  (excludeTags: Set<string>) => (e: MouseEvent) => {
    if (!(e.target instanceof Element)) {
      return;
    }

    if (excludeTags.has(e.target.tagName)) {
      return;
    }

    if (e.target.closest('[data-context-menu-safe="true"]')) {
      return;
    }

    e.preventDefault();
  };
