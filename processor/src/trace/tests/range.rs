use super::{build_trace_from_ops, Felt, FieldElement, Trace, NUM_RAND_ROWS, ONE, ZERO};
use miden_air::trace::{
    chiplets::hasher::HASH_CYCLE_LEN, range::B_RANGE_COL_IDX, AUX_TRACE_RAND_ELEMENTS,
};
use test_utils::rand::rand_array;
use vm_core::{ExtensionOf, Operation};

/// This test checks that range check lookups from stack operations are balanced by the range checks
/// processed in the Range Checker.
///
/// The `U32add` operation results in 4 16-bit range checks of 256, 0, 0, 0.
#[test]
fn b_range_trace_stack() {
    let stack = [1, 255];
    let operations = vec![Operation::U32add];
    let mut trace = build_trace_from_ops(operations, &stack);

    let rand_elements = rand_array::<Felt, AUX_TRACE_RAND_ELEMENTS>();
    let alpha = rand_elements[0];
    let aux_columns = trace.build_aux_segment(&[], &rand_elements).unwrap();
    let b_range = aux_columns.get_column(B_RANGE_COL_IDX);

    assert_eq!(trace.length(), b_range.len());

    // --- Check the stack processor's range check lookups. ---------------------------------------

    // Before any range checks are executed, the value in b_range should be one.
    assert_eq!(ONE, b_range[0]);
    assert_eq!(ONE, b_range[1]);

    // The first range check lookup from the stack will happen when the add operation is executed,
    // at cycle 1. (The trace begins by executing `span`). It must be subtracted out of `b_range`.
    // The range-checked values are 0, 256, 0, 0, so the values to subtract are 3/(alpha - 0) and
    // 1/(alpha - 256).
    let lookups = alpha.inv().mul_base(Felt::new(3)) + (alpha - Felt::new(256)).inv();
    let mut expected = b_range[1] - lookups;
    assert_eq!(expected, b_range[2]);

    // --- Check the range checker's lookups. -----------------------------------------------------

    // 44 rows are needed for 0, 243, 252, 255, 256, ... 38 additional bridge rows of powers of
    // 3 ..., 65535. (0 and 256 are range-checked. 65535 is the max, and the rest are "bridge"
    // values.) An extra row is added to pad the u16::MAX value.
    let len_16bit = 44 + 1;
    // The start of the values in the range checker table.
    let values_start = trace.length() - len_16bit - NUM_RAND_ROWS;

    // After the padded rows, the first value will be unchanged.
    assert_eq!(expected, b_range[values_start]);
    // We include 3 lookups of 0.
    expected += alpha.inv().mul_base(Felt::new(3));
    assert_eq!(expected, b_range[values_start + 1]);
    // Then we have 3 bridge rows between 0 and 255 where the value does not change
    assert_eq!(expected, b_range[values_start + 2]);
    assert_eq!(expected, b_range[values_start + 3]);
    assert_eq!(expected, b_range[values_start + 4]);
    // Then we include 1 lookup of 256, so it should be multiplied by alpha + 256.
    expected += (alpha - Felt::new(256)).inv();
    assert_eq!(expected, b_range[values_start + 5]);

    // --- Check the last value of the b_range column is one --------------------------------------

    let last_row = b_range.len() - NUM_RAND_ROWS - 1;
    assert_eq!(ONE, b_range[last_row]);
}

/// This test checks that range check lookups from memory operations are balanced by the
/// range checks processed in the Range Checker.
///
/// The `StoreW` memory operation results in 2 16-bit range checks of 0, 0.
/// The `LoadW` memory operation results in 2 16-bit range checks of 0, 0.
#[test]
#[allow(clippy::needless_range_loop)]
fn b_range_trace_mem() {
    let stack = [0, 1, 2, 3, 4, 0];
    let operations = vec![
        Operation::MStoreW,
        Operation::Drop,
        Operation::Drop,
        Operation::Drop,
        Operation::Drop,
        Operation::MLoadW,
    ];
    let mut trace = build_trace_from_ops(operations, &stack);

    let rand_elements = rand_array::<Felt, AUX_TRACE_RAND_ELEMENTS>();
    let alpha = rand_elements[0];
    let aux_columns = trace.build_aux_segment(&[], &rand_elements).unwrap();
    let b_range = aux_columns.get_column(B_RANGE_COL_IDX);

    assert_eq!(trace.length(), b_range.len());

    // The memory section of the chiplets trace starts after the span hash.
    let memory_start = HASH_CYCLE_LEN;

    // 40 rows are needed for 0, 3, 4, ... 36 bridge additional bridge rows of powers of
    // 3  ..., 65535. (0 and 4 are both range-checked. 65535 is the max, and the rest are "bridge"
    // values.) An extra row is added to pad the u16::MAX value.
    let len_16bit = 40 + 1;
    let values_start = trace.length() - len_16bit - NUM_RAND_ROWS;

    // The value should start at ONE and be unchanged until the memory processor section begins.
    let mut expected = ONE;
    for row in 0..memory_start {
        assert_eq!(expected, b_range[row]);
    }

    // --- Check the memory processor's range check lookups. --------------------------------------

    // There are two memory lookups. For each memory lookup, the context and address are unchanged,
    // so the delta values indicated the clock cycle change i' - i - 1.
    // StoreW is executed at cycle 1 (after the initial span), so i' - i - 1 = 0.
    let (d0_store, d1_store) = (ZERO, ZERO);
    // LoadW is executed at cycle 6, so i' - i - 1 = 6 - 1 - 1 = 4.
    let (d0_load, d1_load) = (Felt::new(4), ZERO);

    // Include the lookups from the `MStoreW` operation at the next row.
    expected -= (alpha - d0_store).inv() + (alpha - d1_store).inv();
    assert_eq!(expected, b_range[memory_start + 1]);
    // Include the lookup from the `MLoadW` operation at the next row.
    expected -= (alpha - d0_load).inv() + (alpha - d1_load).inv();
    assert_eq!(expected, b_range[memory_start + 2]);

    // The value should be unchanged until the range checker's lookups are included.
    for row in memory_start + 2..=values_start {
        assert_eq!(expected, b_range[row]);
    }

    // --- Check the range checker's lookups. -----------------------------------------------------

    // We include 3 lookups of ZERO in the next row.
    expected += alpha.inv().mul_base(Felt::new(3));
    assert_eq!(expected, b_range[values_start + 1]);

    // then we have one bridge row between 0 and 4 where the value does not change.
    assert_eq!(expected, b_range[values_start + 2]);

    // We include 1 lookup of 4 in the next row.
    expected += (alpha - d0_load).inv();
    assert_eq!(expected, b_range[values_start + 3]);

    // --- The value should now be ONE for the rest of the trace. ---------------------------------
    assert_eq!(expected, ONE);
    for i in (values_start + 3)..(b_range.len() - NUM_RAND_ROWS) {
        assert_eq!(ONE, b_range[i]);
    }
}
