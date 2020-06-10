#[cfg(feature = "bls12_377")]
pub mod bls12_377;

#[cfg(feature = "edwards_on_bls12_377")]
pub mod edwards_on_bls12_377;

#[cfg(feature = "edwards_on_cp6_782")]
pub mod edwards_on_cp6_782;

#[cfg(all(not(feature = "edwards_on_cp6_782"), feature = "edwards_on_bw6_761"))]
pub(crate) mod edwards_on_cp6_782;

#[cfg(feature = "edwards_on_bw6_761")]
pub mod edwards_on_bw6_761;

#[cfg(feature = "edwards_on_bls12_381")]
pub mod edwards_on_bls12_381;

#[cfg(feature = "mnt4_298")]
pub mod mnt4_298;

#[cfg(feature = "mnt4_753")]
pub mod mnt4_753;

#[cfg(feature = "mnt6_298")]
pub mod mnt6_298;

#[cfg(feature = "mnt6_753")]
pub mod mnt6_753;
