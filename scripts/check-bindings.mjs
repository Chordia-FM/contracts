#!/usr/bin/env node
// Drift check for @chordia/contracts: keeps index.ts in sync with the generated ./bindings.
//
// ts-rs generates one ./bindings/<Type>.ts per wire type (run `npm run generate`). index.ts then
// surfaces each to consumers either by re-exporting it (`export * from "./bindings/<Type>"`) or by
// hand-writing an override (`export interface/type <Type>`). For example, GrantResponse pins
// expires_at to `number` instead of ts-rs's `bigint`, and GrantRequest makes fields optional. This
// script fails when a binding is surfaced by neither (a forgotten export, a known footgun) or when
// index.ts re-exports a binding that no longer exists (a stale export after a type was removed or
// renamed).
//
// It does NOT detect a *stale* binding file (one whose Rust source changed but wasn't regenerated);
// CI covers that by running `npm run generate` and asserting `git diff --exit-code` is clean.

import { readdirSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const index = readFileSync(join(root, "index.ts"), "utf8");

const bindings = readdirSync(join(root, "bindings"))
	.filter((f) => f.endsWith(".ts"))
	.map((f) => f.slice(0, -3));

const reExports = [
	...index.matchAll(/export \* from "\.\/bindings\/([A-Za-z0-9_]+)"/g),
].map((m) => m[1]);
const reExported = new Set(reExports);
const localDecls = new Set(
	[...index.matchAll(/export (?:interface|type) ([A-Za-z0-9_]+)\b/g)].map(
		(m) => m[1],
	),
);
const bindingSet = new Set(bindings);

const problems = [];

// Every generated binding must be surfaced from index.ts (re-exported or locally overridden).
for (const name of bindings) {
	if (!reExported.has(name) && !localDecls.has(name)) {
		problems.push(
			`binding ./bindings/${name}.ts is not surfaced from index.ts. Add: export * from "./bindings/${name}";`,
		);
	}
}

// Every re-export must point at a binding that still exists.
for (const name of reExported) {
	if (!bindingSet.has(name)) {
		problems.push(
			`index.ts re-exports ./bindings/${name} but bindings/${name}.ts does not exist (stale export?)`,
		);
	}
}

if (problems.length > 0) {
	console.error(`✗ @chordia/contracts bindings out of sync (${problems.length}):`);
	for (const p of problems) console.error(`  - ${p}`);
	process.exit(1);
}

const overrides = bindings.filter(
	(n) => !reExported.has(n) && localDecls.has(n),
).length;
console.log(
	`✓ ${bindings.length} bindings surfaced from index.ts (${reExported.size} re-exported, ${overrides} hand-written overrides)`,
);
