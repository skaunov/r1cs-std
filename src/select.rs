use crate::prelude::*;
use ark_ff::Field;
use ark_relations::r1cs::{LinearCombination, SynthesisError};
use ark_std::vec::Vec;
/// Generates constraints for selecting between one of many values.
pub trait CondSelectGadget<ConstraintF: Field>
where
    Self: Sized,
    Self: Clone,
{
    /// If `cond == &Boolean::TRUE`, then this returns `true_value`; else,
    /// returns `false_value`.
    ///
    /// # Note
    /// `Self::conditionally_select(cond, true_value, false_value)?` can be more
    /// succinctly written as `cond.select(&true_value, &false_value)?`.
    fn conditionally_select(
        cond: &Boolean<ConstraintF>,
        true_value: &Self,
        false_value: &Self,
    ) -> Result<Self, SynthesisError>;

    /// Returns an element of `values` whose index in represented by `position`.
    /// `position` is an array of boolean that represents an unsigned integer in
    /// big endian order. This is hybrid method 5.3 from https://github.com/mir-protocol/r1cs-workshop/blob/master/workshop.pdf.
    ///
    /// # Example
    /// To get the 6th element of `values`, convert unsigned integer 6 (`0b110`)
    /// to `position = [True, True, False]`,
    /// and call `conditionally_select_power_of_two_vector(position, values)`.
    fn conditionally_select_power_of_two_vector(
        position: &[Boolean<ConstraintF>],
        values: &[Self],
    ) -> Result<Self, SynthesisError> {
        let _ = sum_of_conditions(position, values);
        repeated_selection(position, values)
    }
}

fn count_ones(x: usize) -> usize {
    // count the number of 1s in the binary representation of x
    let mut count = 0;
    let mut y = x;
    while y > 0 {
        count += y & 1;
        y >>= 1;
    }
    count
}

/// Sum of conditions method 5.2 from https://github.com/mir-protocol/r1cs-workshop/blob/master/workshop.pdf
fn sum_of_conditions<ConstraintF: Field, CondG: CondSelectGadget<ConstraintF>>(
    position: &[Boolean<ConstraintF>],
    values: &[CondG],
) -> Result<CondG, SynthesisError> {
    let m = values.len();
    let n = position.len();

    // Assert m is a power of 2, and n = log(m)
    assert!(m.is_power_of_two());
    assert_eq!(1 << n, m);

    let mut selectors: Vec<LinearCombination<ConstraintF>> = Vec::with_capacity(m);

    // fill the selectors vec with Boolean true entries
    for _ in 0..m {
        selectors.push(Boolean::constant(true).lc());
    }

    // let's construct the table of selectors.
    // for a bit-decomposition (b_{n-1}, b_{n-2}, ..., b_0) of `power`:
    // [
    //      (b_{n-1} * b_{n-2} * ... * b_1 * b_0),
    //      (b_{n-1} * b_{n-2} * ... * b_1),
    //      (b_{n-1} * b_{n-2} * ... * b_0),
    //      ...
    //      (b_1 * b_0),
    //      b_1,
    //      b_0,
    //      1,
    // ]
    // signal selectors[leafCount];
    //
    // the element of the selector table at index i is a product of `bits`
    // e.g. for i = 5 == (101)_binary
    // `selectors[5]` <== b_2 * b_0`
    // we can construct the first `max_bits_in_power - 1` elements without products,
    // directly from `bits`:
    // e.g. for
    // `selectors[1] <== b_0`
    // `selectors[2] <== b_1`
    // `selectors[4] <== b_2`
    // `selectors[8] <== b_3`

    // First element is true, but we've already filled it in.
    // selectors[0] = Boolean::constant(true);
    for i in 0..n {
        selectors[1 << i] = position[i].lc();
        for j in (1 << i) + 1..(1 << (i + 1)) {
            selectors[j] = &selectors[1 << i] + &selectors[j - (1 << i)];
        }
    }

    let mut selector_sums: Vec<LinearCombination<ConstraintF>> = Vec::with_capacity(m);
    for i in 0..m {
        for j in 0..m {
            if i | j == j {
                let counts = count_ones(j - i);
                if counts % 2 == 0 {
                    selector_sums[i] = &selector_sums[i] + &selectors[j];
                } else {
                    selector_sums[i] = &selector_sums[i] - &selectors[j];
                };
            }
        }
    }

    let root: LinearCombination<ConstraintF> = LinearCombination::zero();
    // var x = 0;
    for i in 0..m {
        root = &root + (values[i], selector_sums[i]);
    }
    // for (var i = 0; i < nextPow; i++) {
    //     x += leaves[i] * selector_sums[i];
    // }
    // root <== x;

    unimplemented!()
}

/// Repeated selection method 5.1 from https://github.com/mir-protocol/r1cs-workshop/blob/master/workshop.pdf
fn repeated_selection<ConstraintF: Field, CondG: CondSelectGadget<ConstraintF>>(
    position: &[Boolean<ConstraintF>],
    values: &[CondG],
) -> Result<CondG, SynthesisError> {
    let m = values.len();
    let n = position.len();

    // Assert m is a power of 2, and n = log(m)
    assert!(m.is_power_of_two());
    assert_eq!(1 << n, m);

    let mut cur_mux_values = values.to_vec();

    // Traverse the evaluation tree from bottom to top in level order traversal.
    for i in 0..n {
        // Size of current layer.
        let cur_size = 1 << (n - i);
        assert_eq!(cur_mux_values.len(), cur_size);

        let mut next_mux_values = Vec::new();
        for j in (0..cur_size).step_by(2) {
            let cur = CondG::conditionally_select(
                &position[n - 1 - i],
                // true case
                &cur_mux_values[j + 1],
                // false case
                &cur_mux_values[j],
            )?;
            next_mux_values.push(cur);
        }
        cur_mux_values = next_mux_values;
    }

    Ok(cur_mux_values[0].clone())
}

/// Performs a lookup in a 4-element table using two bits.
pub trait TwoBitLookupGadget<ConstraintF: Field>
where
    Self: Sized,
{
    /// The type of values being looked up.
    type TableConstant;

    /// Interprets the slice `bits` as a two-bit integer `b = bits[0] + (bits[1]
    /// << 1)`, and then outputs `constants[b]`.
    ///
    /// For example, if `bits == [0, 1]`, and `constants == [0, 1, 2, 3]`, this
    /// method should output a variable corresponding to `2`.
    ///
    /// # Panics
    ///
    /// This method panics if `bits.len() != 2` or `constants.len() != 4`.
    fn two_bit_lookup(
        bits: &[Boolean<ConstraintF>],
        constants: &[Self::TableConstant],
    ) -> Result<Self, SynthesisError>;
}

/// Uses three bits to perform a lookup into a table, where the last bit
/// conditionally negates the looked-up value.
pub trait ThreeBitCondNegLookupGadget<ConstraintF: Field>
where
    Self: Sized,
{
    /// The type of values being looked up.
    type TableConstant;

    /// Interprets the slice `bits` as a two-bit integer `b = bits[0] + (bits[1]
    /// << 1)`, and then outputs `constants[b] * c`, where `c = if bits[2] {
    /// -1 } else { 1 };`.
    ///
    /// That is, `bits[2]` conditionally negates the looked-up value.
    ///
    /// For example, if `bits == [1, 0, 1]`, and `constants == [0, 1, 2, 3]`,
    /// this method should output a variable corresponding to `-1`.
    ///
    /// # Panics
    ///
    /// This method panics if `bits.len() != 3` or `constants.len() != 4`.
    fn three_bit_cond_neg_lookup(
        bits: &[Boolean<ConstraintF>],
        b0b1: &Boolean<ConstraintF>,
        constants: &[Self::TableConstant],
    ) -> Result<Self, SynthesisError>;
}
