import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';

const revision = '36e584d59152181927e0d8bf70fddcd2a5f33d6c';
const source = process.env.PELE_SOURCE;
if (!source) throw new Error('Set PELE_SOURCE to the pinned Pele checkout with npm dependencies installed.');
const actual = execFileSync('git', ['-C', source, 'rev-parse', 'HEAD'], { encoding: 'utf8' }).trim();
if (actual !== revision) throw new Error(`Expected Pele ${revision}, found ${actual}.`);
const { build } = await import(pathToFileURL(path.join(source, 'node_modules/esbuild/lib/main.js')).href);
const bundled = await build({
  absWorkingDir: source,
  stdin: {
    contents: "export * from './src/layout/compound.ts'; export { wrapComponents } from './src/layout/components.ts';",
    resolveDir: source,
    loader: 'ts',
  },
  bundle: true,
  platform: 'node',
  format: 'esm',
  write: false,
});
const reference = await import(`data:text/javascript;base64,${Buffer.from(bundled.outputFiles[0].text).toString('base64')}`);
const seed = 987654321;
let state = seed;
const next = () => (state = (Math.imul(state, 1664525) + 1013904223) >>> 0);
const pick = (list) => list[next() % list.length];
const DIRS = ['TB', 'BT', 'LR', 'RL'];
const cases = [];
for (let k = 0; k < 160; k++) {
  const big = k % 10 === 9;
  const leaves = (big ? 14 : 2) + (next() % (big ? 16 : 10));
  const groupCount = k % 8 === 0 ? 0 : 1 + (next() % (big ? 6 : 4));
  const mixed = k % 3 === 0;
  const n = leaves + groupCount;
  const isGroup = Array.from({ length: n }, () => false);
  const groupIds = [];
  while (groupIds.length < groupCount) {
    const i = next() % n;
    if (!isGroup[i]) {
      isGroup[i] = true;
      groupIds.push(i);
    }
  }
  const nodes = [];
  for (let i = 0; i < n; i++) {
    let c;
    if (isGroup[i]) {
      c = reference.cnode(0, 0);
      c.isGroup = true;
      c.dir = mixed && next() % 2 === 0 ? pick(DIRS) : undefined;
      c.padX = k % 4 === 1 ? 12 : 20;
      c.padTop = 20 + (next() % 3) * 12;
      c.padBottom = c.padX;
      c.minW = next() % 3 === 0 ? 40 + (next() % 160) : 0;
      c.span = Infinity;
    } else {
      c = reference.cnode(30 + (next() % 100), 20 + (next() % 45));
      c.span = next() % 4 === 0 ? 0 : Math.max(0, c.w - 24);
    }
    c.seq = k % 6 === 5 ? next() % n : i;
    nodes.push(c);
  }
  for (let i = 0; i < n; i++) {
    if (groupIds.length === 0 || next() % 3 === 0) continue;
    const g = pick(groupIds);
    if (g === i) continue;
    nodes[i].parent = g;
  }
  if (k % 25 === 7 && k % 4 !== 2 && groupIds.length > 0) nodes[0].parent = n + 3;
  const edges = [];
  const edgeCount = 1 + (next() % (big ? 3 * n : 2 * n));
  for (let j = 0; j < edgeCount; j++) {
    const src = next() % n;
    let dst = next() % n;
    if (src === dst && next() % 4 !== 0) dst = (dst + 1) % n;
    const labelled = next() % 4 === 0;
    edges.push({
      src,
      dst,
      minlen: next() % 6 === 0 ? 2 : 1,
      labelW: labelled ? 20 + (next() % 60) : 0,
      labelH: labelled ? 22 : 0,
      route: [],
      labelX: 0,
      labelY: 0,
    });
  }
  const rootDir = pick(DIRS);
  const inherit = k % 4 === 2;
  const opt = {
    nodeSep: k % 5 === 4 ? 20 : 40,
    edgeSep: 16,
    rankSep: k % 5 === 4 ? 36 : 48,
    portSep: 20,
    tracks: k % 5 === 1,
    straight: k % 7 === 3,
    headRoom: 7,
  };
  const initial = { nodes: structuredClone(nodes), edges: structuredClone(edges), rootDir, opt, inherit };
  if (inherit) reference.inheritCrossedDirections(nodes, edges.map((e) => [e.src, e.dst]));
  const result = reference.compoundLayout(nodes, edges, rootDir, opt);
  const placed = () => ({
    nodes: nodes.map(({ x, y, w, h, parent, seq, dir }) => ({ x, y, w, h, parent, seq, dir })),
    edges: edges.map(({ src, dst, route, labelX, labelY }) => ({ src, dst, route: route.slice(), labelX, labelY })),
  });
  const expected = { result, ...placed() };
  let wrap;
  if (k % 3 === 1) {
    const width = Math.max(1, result.width * (0.3 + (next() % 5) / 10));
    const gap = 40;
    const wrapped = reference.wrapComponents(nodes, edges, width, gap);
    wrap = { width, gap, result: wrapped ?? null, ...(wrapped ? placed() : {}) };
  }
  cases.push({ ...initial, expected, wrap });
}
const replacer = (_key, value) => (typeof value === 'number' && !Number.isFinite(value) ? String(value) : value);
const destination = fileURLToPath(new URL('../tests/fixtures/compound-reference.json', import.meta.url));
fs.writeFileSync(destination, JSON.stringify({ upstreamRevision: revision, seed, cases }, replacer) + '\n');
console.log(`Wrote ${cases.length} compound reference cases to ${destination}`);
