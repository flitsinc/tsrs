# Deduplicate checker diagnostics before arena allocation

The public 1,050-message reproduction in [TypeScript #65095](https://github.com/microsoft/TypeScript/issues/65095)
retains millions of duplicate instantiation-limit diagnostics in tsrs's thread arenas. The checker discards them
logically, but arena allocation keeps their storage until the arena is reclaimed. Constructing an owned temporary
and checking the existing diagnostic collection before arena allocation removes this retention.

## Controlled result

macOS arm64, Rust 1.99.1, normal release profile and mimalloc, identical inputs and compiler options. Stock and
candidate use tsrs `48cf809ef3fc59e799d02872b247477e6c7d1c25`, pinned to TypeScript beta
`3e9236361fa97a9e6c685f4833ac457ae2d8c50a`. The candidate adds only the diagnostic-allocation implementation;
the landing version additionally exposes existing internals for integration tests. Three uncapped samples per
Rust reference-mode configuration, interleaved after the initial stock samples:

| Reference mode | Stock peak RSS median | Candidate peak RSS median | Stock instructions median | Candidate instructions median |
| --- | --- | --- | --- | --- |
| 1 checker | 3.4446 GiB | 1.0801 GiB | 40,471,747,392 | 37,603,296,936 |
| 4 checkers | 3.4484 GiB | 1.0835 GiB | 40,470,061,760 | 37,537,234,776 |

The one-checker RSS ranges are 3.4267–3.4456 GiB before and 1.0798–1.0806 GiB after. Four-checker ranges are
3.4477–3.4486 GiB before and 1.0826–1.0840 GiB after. Peak RSS falls about 69%; retired instructions fall about
7%. This busy machine does not provide reliable wall-time comparisons, so no speedup is claimed.

Go observations on the same input (one sample per cell, not paired performance estimates):

| Go revision | 1-checker peak RSS | 4-checker peak RSS |
| --- | --- | --- |
| beta `3e9236361fa9` | 2.6708 GiB | 2.7519 GiB |
| main `e2bc2bcb838e` | 2.5921 GiB | 2.5197 GiB |

Go beta already finishes below 3 GiB. The 79 upstream commits between these pins are therefore unnecessary to
make Go fit this case. No upstream memory commit is included in this fix. Go main used its committed PGO profile;
the beta build disabled PGO. These observations do not isolate the small Go before/after difference to commits.

All completed measurements produce the same complete diagnostic text, in order: TS2345, TS2590, TS2345, TS2589.
The diagnostic-only SHA256 is `a754ba2751756d59ab4c558d11c3714e7030ee42184666ac161836244925bbde`.
The fixture uses next-intl/use-intl 4.13.4, @types/react 19.2.2 and integrity-checked declaration dependencies;
all imports resolve. Its input manifest contains 303 files, 1,705,716 bytes and 1,050 message leaves.

## Attribution and controls

`TSRS_MEM_SPLIT=1` reports 3,339.0 MiB of used thread-arena storage before the fix and 918.2 MiB after it, with
88.9 MiB live heap in both cases and negligible unused resident arena storage. The thread-arena allocation-site
sampler attributes about 2,422 MiB to `new_diagnostic -> Checker::error -> instantiate_type_with_alias_worker`.
This is retained diagnostic storage, rather than a heap allocator or checker-duplication explanation.

One-checker counts remain exactly 8,631,208 types, 5,511,719 instantiations and 61,179 symbols before/after.
Go beta and main both report 8,631,216 types and 5,511,719 instantiations. The eight-type cross-language difference
predates the fix. Four-checker tsrs counts also remain unchanged (8,631,463 types; 5,511,719 instantiations).

Single-sample stock controls keep the same diagnostic hash:

- `--lazyMembers`: 3.4449 GiB peak RSS.
- `--fast`: 3.5023 GiB peak RSS.
- System allocator: 3.4408 GiB peak RSS (sampler compiled but disabled at runtime).

Neither the optional member machinery nor allocator selection explains this retention. No broad memory bound,
catalog-wide improvement, Linux result or language-server retention claim follows from this one public CLI case.

## Implementation and correctness

`DiagnosticsCollection::add_new` constructs a normal owned `Diagnostic`, checks the same location key and full
equality as `add`, then allocates only a distinct diagnostic. Duplicate argument strings drop normally. It returns
the existing canonical pointer, preserving subsequent related-information mutations. Already-built diagnostics
continue to use the existing `add` API. A miss deliberately delegates to `add`, repeating its lookup rather than
introducing a second insertion implementation; a workload with mostly distinct diagnostics may pay that cost.

The existing diagnostic escape guard covers temporary construction and insertion, so accepted diagnostics keep
their routed owner through nested checker/file scratch teardown. The checker still increments diagnostic attempts
once per call, including duplicates and suppressed calls. At the serialization limit it still returns a fresh,
uncollected diagnostic, preserving independent mutation and pointer identity.

Integration coverage checks zero additional arena bytes for 10,000 duplicate reports, collisions and arguments,
canonical pointer reuse and related information, nested scratch survival with and without diagnostic routing,
and checker attempt counting plus serialization suppression. The AST, checker and compiler integration suites
pass (34 + 21 + 31 tests); their all-target Clippy run with warnings denied, source lint and formatting are clean.
Two independent read-only implementation reviews found no blockers. Full merged conformance and catalog checks
belong to the catchup integration gate; this note does not claim they have run on the combined branch.

## Reproduction

Use the issue's fixture with exactly 1,050 generated messages and the declaration versions above. Compare release
binaries from the same base and pin. From the fixture directory, run each configuration uncapped, retaining complete
output and `/usr/bin/time -l` counters; alternate stock/candidate samples and match checker counts:

```sh
TSRS_MEM_SPLIT=1 /usr/bin/time -l /path/to/tsrs -p . --pretty false --extendedDiagnostics --checkers 1
TSRS_MEM_SPLIT=1 /usr/bin/time -l /path/to/tsrs -p . --pretty false --extendedDiagnostics --checkers 4
```

Exit status 2 is expected because the reproduction intentionally reports the four diagnostics. Compare the complete
diagnostic streams, excluding extended statistics, rather than only their codes or success status. The landing
branch uses the catchup base pinned to `e2bc2bcb838e60fd44c02f0b224b8201a4d7ff5b`; the controlled beta pair above
isolates the allocation change from that pin update and from other catchup ports.

Final landing-source confirmation (catchup `c3a0dd17` plus the final implementation/tests, exact e2bc reference
pin, one uncapped sample per cell): reference mode peaks at 1.0821 GiB / 37,787,925,584 retired instructions with
one checker and 1.0824 GiB / 37,618,747,628 instructions with four checkers. At one checker, `--lazyMembers` peaks
at 1.0808 GiB and `--fast` at 1.1303 GiB. All four retain the same diagnostic hash above. These confirmations
are separate from the controlled beta medians and include the catchup base's other changes.
