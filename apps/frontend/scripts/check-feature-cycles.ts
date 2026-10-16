// Fails when features import each other in a cycle, type-only imports included.
// dependency-cruiser's no-circular works on files: a back edge between two features is
// missed unless the chain returns to the same file. Here the graph is collapsed to one
// node per feature first. See ARCHITECTURE.md.

import { execFileSync } from 'node:child_process';

type Module = { source: string; dependencies: { resolved: string }[] };

const FEATURE = /^src\/features\/[^/]+/;

// process.execPath is the bun running this script, so nothing is looked up in PATH.
const output = execFileSync(
  process.execPath,
  [
    'x',
    'depcruise',
    'src',
    '--config',
    '.dependency-cruiser.cjs',
    '--include-only',
    '^src/features/',
    '--collapse',
    FEATURE.source,
    '--output-type',
    'json',
  ],
  { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 },
);
const { modules } = JSON.parse(output) as { modules: Module[] };

const name = (path: string) => FEATURE.exec(path)?.[0];
const graph = new Map<string, Set<string>>();

for (const module of modules) {
  const from = name(module.source);

  if (!from) {
    continue;
  }

  const targets = graph.get(from) ?? new Set<string>();

  for (const { resolved } of module.dependencies) {
    const to = name(resolved);

    if (to && to !== from) {
      targets.add(to);
    }
  }

  graph.set(from, targets);
}

// Tarjan's strongly connected components: every component of two or more features is a cycle.
const index = new Map<string, number>();
const low = new Map<string, number>();
const stack: string[] = [];
const onStack = new Set<string>();
const cycles: string[][] = [];

const visit = (node: string) => {
  index.set(node, index.size);
  low.set(node, index.get(node)!);
  stack.push(node);
  onStack.add(node);

  for (const next of graph.get(node) ?? []) {
    if (!index.has(next)) {
      visit(next);
      low.set(node, Math.min(low.get(node)!, low.get(next)!));
    } else if (onStack.has(next)) {
      low.set(node, Math.min(low.get(node)!, index.get(next)!));
    }
  }

  if (low.get(node) === index.get(node)) {
    const component: string[] = [];
    let member: string;

    do {
      member = stack.pop()!;
      onStack.delete(member);
      component.push(member);
    } while (member !== node);

    if (component.length > 1) {
      cycles.push(component.sort());
    }
  }
};

for (const node of graph.keys()) {
  if (!index.has(node)) {
    visit(node);
  }
}

if (cycles.length > 0) {
  for (const cycle of cycles) {
    const edges = cycle.flatMap((from) =>
      [...(graph.get(from) ?? [])]
        .filter((to) => cycle.includes(to))
        .map((to) => `  ${from} → ${to}`),
    );
    process.stderr.write(
      `feature cycle: ${cycle.join(', ')}\n${edges.join('\n')}\n`,
    );
  }

  process.stderr.write(
    '\nFeatures import each other one way only. Move what both sides need into the base.\n',
  );
  process.exit(1);
}

process.stdout.write(`✔ no feature cycles (${graph.size} features)\n`);
