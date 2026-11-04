#!/usr/bin/env node
// Retain the pinned upstream locale artifacts, including their generated runtime archives.
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { ROOT, args, main, pinnedCommit, requireValue } from "../lib/cli.mjs";

main(() => {
    args("Copy generated locale catalogs and archives from the pinned TypeScript reference.");
    const reference = path.join(ROOT, "ts-ref");
    const head = execFileSync("git", ["-C", reference, "rev-parse", "HEAD"], { encoding: "utf8" }).trim();
    requireValue(head === pinnedCommit(), "ts-ref does not match the workspace TypeScript pin");
    execFileSync("git", ["-C", reference, "diff", "--exit-code", "HEAD", "--", "tsc/internal/diagnostics/loc"]);
    const source = path.join(reference, "tsc/internal/diagnostics/loc");
    const target = path.join(ROOT, "crates/tsrs_diagnostics/loc");
    const names = fs.readdirSync(source).filter(name => name.endsWith(".generated.json")).sort();
    requireValue(names.length === 13, "expected all thirteen upstream locale catalogs");
    for (const name of names) {
        fs.copyFileSync(path.join(source, name), path.join(target, name));
        const archive = name.replace(/\.generated\.json$/, ".json.gz");
        fs.copyFileSync(path.join(source, archive), path.join(target, archive));
    }
    console.log(`copied ${names.length} generated locale pairs from ${head}`);
});
