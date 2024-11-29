// Copyright 2024, Horizen Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::CurveHooks;

use ark_bn254::g1::Config as ArkConfig;
use ark_models_ext::{bn, short_weierstrass::SWCurveConfig, CurveConfig};
use ark_std::marker::PhantomData;

pub use ark_bn254::g1::{G1_GENERATOR_X, G1_GENERATOR_Y};

pub type G1Affine<H> = bn::G1Affine<crate::Config<H>>;
pub type G1Projective<H> = bn::G1Projective<crate::Config<H>>;

#[derive(Clone, Copy)]
pub struct Config<H: CurveHooks>(PhantomData<fn() -> H>);

impl<H: CurveHooks> CurveConfig for Config<H> {
    const COFACTOR: &'static [u64] = <ArkConfig as CurveConfig>::COFACTOR;
    const COFACTOR_INV: Self::ScalarField = <ArkConfig as CurveConfig>::COFACTOR_INV;

    type BaseField = <ArkConfig as CurveConfig>::BaseField;
    type ScalarField = <ArkConfig as CurveConfig>::ScalarField;
}

impl<H: CurveHooks> SWCurveConfig for Config<H> {
    const COEFF_A: Self::BaseField = <ArkConfig as SWCurveConfig>::COEFF_A;
    const COEFF_B: Self::BaseField = <ArkConfig as SWCurveConfig>::COEFF_B;

    const GENERATOR: G1Affine<H> = G1Affine::<H>::new_unchecked(G1_GENERATOR_X, G1_GENERATOR_Y);

    /// Multi scalar multiplication jumping into the user-defined `msm_g1` hook.
    ///
    /// On any internal error returns `Err(0)`.
    #[inline(always)]
    fn msm(bases: &[G1Affine<H>], scalars: &[Self::ScalarField]) -> Result<G1Projective<H>, usize> {
        if bases.len() != scalars.len() {
            return Err(bases.len().min(scalars.len()));
        }
        H::bn254_msm_g1(bases, scalars).map_err(|_| 0)
    }

    /// Projective multiplication jumping into the user-defined `mul_projective` hook.
    ///
    /// On any internal error returns `Projective::zero()`.
    #[inline(always)]
    fn mul_projective(base: &G1Projective<H>, scalar: &[u64]) -> G1Projective<H> {
        H::bn254_mul_projective_g1(base, scalar).unwrap_or_default()
    }

    /// Affine multiplication jumping into the user-defined `mul_projective` hook.
    ///
    /// On any internal error returns `Projective::zero()`.
    #[inline(always)]
    fn mul_affine(base: &G1Affine<H>, scalar: &[u64]) -> G1Projective<H> {
        Self::mul_projective(&(*base).into(), scalar)
    }

    #[inline(always)]
    fn mul_by_a(elem: Self::BaseField) -> Self::BaseField {
        <ArkConfig as SWCurveConfig>::mul_by_a(elem)
    }

    // Verbatim copy of upstream implementation.
    //
    // Can't call it directly because of different `Affine` configuration.
    #[inline(always)]
    fn is_in_correct_subgroup_assuming_on_curve(_: &G1Affine<H>) -> bool {
        // G1 = E(Fq) so if the point is on the curve, it is also in the subgroup.
        true
    }
}
