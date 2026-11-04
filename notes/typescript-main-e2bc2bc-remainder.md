# TypeScript main catch-up: remainder ports

The unreleased fork targets `microsoft/TypeScript@e2bc2bcb838e60fd44c02f0b224b8201a4d7ff5b`.
This note covers Port R's 41 commits in the 79-commit interval from `3e9236361fa9`.
The integration branch also carries the independent semantic, lazy-member, project/LS and SDK ports.

## Applied changes

| Upstream | Rust implementation |
| --- | --- |
| `98add1bbed1c` #64898 | Indexed original-root membership for no-op builds; all 29 virtual-path assertions and resolved-root case. |
| `1f82f368b668` #64972 | Pre-size printer output only for whole files. |
| `34285337bd4c` #64979 | Insertion-ordered symbol tables and corresponding checker, binder, printer and declaration-order consumers. |
| `df33c6a9de84` #64976 | Qualify merged namespace references in import-equals emit. |
| `46b80be4dc34` #64826 | Refresh retained localization catalogs and archives from the pin. |
| `292e4f208fd5` #64869 | Use the existing symbol-id link store for merged symbols, including shared-graph copy-on-write forks. |
| `1f6d8e331c4f` #64992 | Retain Rust's existing compile-time keyword perfect hash; add upstream exhaustive keyword/nonkeyword tests. |
| `0ccce9dcf276` #65045 | Reject non-access flow-reference targets early. |
| `af1575cf2ee9` #64868 | Compare reference names before symbols. |
| `9973e1c8b56b` #65060 | Parenthesize new-expression callees. |
| `f995d487a1f3` #64922 | Emit each checker's files in source order. |
| `4e5836fb6c09` #65091 | Fill the config cycle into TS18000. |
| `ba6c27b2f455` #64712 | Incremental option invalidation; generator reads the moved flags from pinned `options.ts`. |
| `988d468c21c8` #65103 | Supply missing diagnostic formatting arguments. |
| `e8bd19f8b4ca` #65116 | Preserve typed string arrays and JSON arrays, including paths entries, in build-info cache conversion. |
| `cf567eb6707f` #64659 | Preserve cached changed-signature results; cover overlapping global invalidation and changed default libraries. Rust already batches dependency-frontier signatures on the checker pool. |
| `1b86397ef565` #65063 | Regenerate AST/codecs, split case/default consumers, synthesize an omitted arrow token, update oracle tooling and refresh codec goldens. |
| `2d7625744f33` #65073 | Compute CommonJS indicator in the parser while preserving binder symbol creation. |
| `f0accfdfe02c` #65093 | Use ordered symbol tables for lazy member layouts after #64915/#65059; reuse scratch table/base indices and test clone/clear independence. |
| `9286e1733771` #64732 | Inline JSDoc links in optional type positions. |
| `ce22b000c6c7` #65279 | Erase types from preserved const-enum initializers. |
| `bc4b3037a731` #64982 | Erase decorated abstract properties with legacy decorators and define semantics. |
| `9aaf14873ad3` #65129 | Remove unused recursive arguments. |
| `b5f1effdc237` #65289 | Generate and honor `disableDynamicImportExtensionRewriting`. |
| `e2bc2bcb838e` #65297 | Preserve single evaluation of wrapped compound-assignment targets. |
| `b2cfccf25228` #65050 | Refresh end-pin locale handbacks in the shared localization regeneration. |

The #64869 representation change is isolated as `d0a60a59`, directly after `5e1f4f3d`.
The integrator owns Linux RSS and instruction-count comparisons against that parent. No performance
improvement is claimed from source review. Other memory-sensitive structures retain the fork's arena,
compressed-pointer and shared-graph conventions.

## Existing equivalents and pinned fixtures

- `97aedd3b75e8` #64964: `is_type_subset_of_union` already chooses sorted scanning or binary search over a shrinking tail; `is_sorted_subset` implements the scan.
- `dd8aad0232fb` #64791: Rust discriminants are encapsulated; import declarations already rebuild through `clone_as_import_declaration`. The affected `importTag5` fixture passes.
- `cb5f3462563e` #65031: arena growth already uses chunk capacity (`end - start`), with the fork's region growth policy.
- `f2c93ea86827` #65024: workspace Clippy requires unsafe documentation; the source gate inventories unsafe sites.
- `6161772c1370` #65067: computed-name state is inline `Cell<Option<bool>>`, avoiding Go's allocated boolean.
- `050fb369ee1f` #64748 and `cc8253043e2f` #64969: the pinned `augmentExportEquals2` and `narrowingNoInfer2` cases run directly through the Rust compiler harness.
- `96cbb3af6c84` #65012: the catch-up base already regenerated bundled es2026 libraries; `jsonParseWithSource` runs through the Rust harness.

## Changes without a supported Rust counterpart

- `538671d3cc2d` #64887: upstream release-tag handling.
- `fe95f01c99bd` #64926: upstream Copilot prose.
- `4a4b24d5dc96` #64400: upstream Hereby/Go generation wiring; Rust uses its own generators and `gen-check`.
- `25f823dc198a` #64986: Go PGO profile data; Rust owns separate training.
- `b7cac4f9512e` #64858: upstream-specific CodeQL/Scorecard action dependencies.
- `02ae10dee1a4` #65027: the deliberately unported Windows `ReadDirectoryChangesW` backend.
- `df2049ceb07d` #65119: upstream test-authoring prose.

## Validation scope

The selected compiler/conformance set contains 47 configured variants from all 37 fixture stems R owns.
Both default and fast runs match all available errors, types, symbols, JavaScript/declarations and source-map
baselines: 47 errors, 29 types, 29 symbols, 47 emit, one JavaScript map and one source-map record per mode.
Missing upstream baseline kinds are reported as skips, not passes.

The pinned Go execute recorder ran in an isolated source copy. All 265 execute baselines touched by R were
selected and replayed: 261 match exactly, with no crashes, missing cases or unexpected differences. Four
comparisons cover unsupported output: tsrs's own `--help --all` text and three `generateTrace` cases.
The execute harness reports differences without failing its test, so the retained verifier additionally
checks the exact selection and rejects every failure outside those four explicit unsupported cases.

Codec goldens come from the source-built e2bc server and the exact published
`typescript@7.1.0-dev.20261104.1` client (`gitHead=e2bc2bc`). All 16 active codec tests pass, including the
real JavaScript client decoder; one existing test is ignored. The module oracle was rebuilt in the isolated
source copy, and its 37 active tests pass with one existing ignored test. All thirteen retained locale pairs
pass formatting, placeholder and archive-content validation; localization runtime remains unsupported.

Full merged conformance, the scheduling sweep, complete Fourslash/API gates, published-version parity and
performance evidence belong to the integrator. These targeted receipts do not establish those broader claims.
