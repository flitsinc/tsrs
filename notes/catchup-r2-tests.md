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
