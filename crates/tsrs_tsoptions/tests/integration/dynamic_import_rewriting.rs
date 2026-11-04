use tsrs_core::{CompilerOptions, Tristate};
use tsrs_tsoptions::{compiler_options_affect_emit, compiler_options_affect_semantic_diagnostics};

#[test]
fn dynamic_import_rewriting_option_invalidates_diagnostics_and_emit() {
    let before = CompilerOptions {
        disable_dynamic_import_extension_rewriting: Tristate::False,
        ..Default::default()
    };
    let after = CompilerOptions {
        disable_dynamic_import_extension_rewriting: Tristate::True,
        ..Default::default()
    };
    assert!(compiler_options_affect_semantic_diagnostics(Some(&before), Some(&after)));
    assert!(compiler_options_affect_emit(Some(&before), Some(&after)));
    assert!(!compiler_options_affect_semantic_diagnostics(Some(&after), Some(&after.clone())));
    assert!(!compiler_options_affect_emit(Some(&after), Some(&after.clone())));
}
