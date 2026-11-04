# Catch-up regression coverage

Reference: microsoft/TypeScript `e2bc2bcb838e60fd44c02f0b224b8201a4d7ff5b`.
The reference checkout supplies these compiler fixtures directly to `tsrs-test`;
they are not duplicated into this repository.

## Additional compiler coverage (#64907)

Upstream `c13270d5fc85c9e9546aa9e3befd4974ededf39b` changes only tests and baselines.
Its fixture set is:

- `ambientConstInitializerNonEnumReference`
- `awaitOperandContextualTypeUnwrapsAwaited`
- `bigintZeroIsDefinitelyFalsy`
- `classExtendsNullValuedIdentifier`
- `identicalSignaturesWithTypePredicates`
- `importHelpersMissingMetadataHelper`
- `jsExpandoUniqueSymbolDeclarationEmit`
- `moduleExportsSelfReferenceCircularity`
- `neverIndexSignatureNotDuplicate`
- `nonexistentPropertyOnThenable`
- `nullishAndLogicalOrRightOperandWithBindingPattern`
- `spreadArgumentWithSingleGenericSignature`
- `staticPropertyUsedBeforeInitializationWithStaticBlocks`
- `uniqueSymbolPropertyNameFromReexportedModule`

Validation receipts are recorded separately from the upstream fixture inventory.

## Protected optional property completions (#65055)

Upstream `9fa923a8446b2f4b141b2e255802b70e0de41d6c` changes only
`TestCompletionsOverridingProperties1`. The pinned Go fourslash harness executes
that test against the Rust language server, including its optional protected
property cases. No runtime port or copied fixture is needed.
