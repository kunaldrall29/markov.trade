#!/usr/bin/env node
// Regenerate `src/generated/*` from the IDL snapshots in `docs/idl/`.
//
// The snapshots are the ones FACTS records as the deployed programs' IDLs;
// nothing here is typed from memory. Run `pnpm generate` after an IDL changes
// and commit the output — CI compares the tree to a fresh run.
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createFromRoot } from "codama";
import { rootNodeFromAnchor } from "@codama/nodes-from-anchor";
import { renderVisitor } from "@codama/renderers-js";

const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "..");
const repo = join(pkg, "..", "..");

const programs = [
  { idl: "docs/idl/mandate-25CdYaZe.json", out: "src/generated/mandate" },
  { idl: "docs/idl/demo-perps-3Zcd8Xs.json", out: "src/generated/demoPerps" },
];

for (const { idl, out } of programs) {
  const json = JSON.parse(readFileSync(join(repo, idl), "utf8"));
  const codama = createFromRoot(rootNodeFromAnchor(json));
  await codama.accept(
    renderVisitor(pkg, {
      generatedFolder: out,
      deleteFolderBeforeRendering: true,
      formatCode: true,
      syncPackageJson: false,
      kitImportStrategy: "rootOnly",
      prettierOptions: { printWidth: 100, singleQuote: false, trailingComma: "all" },
    }),
  );
  console.log(`generated ${out} from ${idl} (${json.address})`);
}
