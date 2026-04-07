export const createPreventRightClick =
  (excludeTags: Set<string>) => (e: MouseEvent) => {
    if (!(e.target instanceof HTMLElement)) {
      return;
    }

    if (excludeTags.has(e.target.tagName)) {
      return;
    }

    e.preventDefault();
  };
