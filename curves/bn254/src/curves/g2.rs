// Copyright 2022 arkworks contributors
// Copyright 2024 Horizen Labs, Inc.
// SPDX-License-Identifier: Apache-2.0 or MIT

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

use ark_bn254::{fq2::Fq2, g2::Config as ArkConfig};
use ark_ff::{Field, MontFp};
use ark_models_ext::{bn, short_weierstrass::SWCurveConfig, AffineRepr, CurveConfig};
use ark_std::marker::PhantomData;

use crate::CurveHooks;

pub use ark_bn254::g2::{
    G2_GENERATOR_X, G2_GENERATOR_X_C0, G2_GENERATOR_X_C1, G2_GENERATOR_Y, G2_GENERATOR_Y_C0,
    G2_GENERATOR_Y_C1,
};

// PSI_X = (u+9)^((p-1)/3) = TWIST_MUL_BY_Q_X
const P_POWER_ENDOMORPHISM_COEFF_0: Fq2 = Fq2::new(
    MontFp!("21575463638280843010398324269430826099269044274347216827212613867836435027261"),
    MontFp!("10307601595873709700152284273816112264069230130616436755625194854815875713954"),
);

// PSI_Y = (u+9)^((p-1)/2) = TWIST_MUL_BY_Q_Y
const P_POWER_ENDOMORPHISM_COEFF_1: Fq2 = Fq2::new(
    MontFp!("2821565182194536844548159561693502659359617185244120367078079554186484126554"),
    MontFp!("3505843767911556378687030309984248845540243509899259641013678093033130930403"),
);

// Integer representation of 6x^2 = t - 1
const SIX_X_SQUARED: [u64; 2] = [17887900258952609094, 8020209761171036667];

pub type G2Affine<H> = bn::G2Affine<crate::Config<H>>;
pub type G2Projective<H> = bn::G2Projective<crate::Config<H>>;

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

    const GENERATOR: G2Affine<H> = G2Affine::<H>::new_unchecked(G2_GENERATOR_X, G2_GENERATOR_Y);

    /// Multi scalar multiplication jumping into the user-defined `msm_g2` hook.
    ///
    /// On any *external* error returns `Err(0)`.
    #[inline(always)]
    fn msm(bases: &[G2Affine<H>], scalars: &[Self::ScalarField]) -> Result<G2Projective<H>, usize> {
        if bases.len() != scalars.len() {
            return Err(bases.len().min(scalars.len()));
        }
        H::bn254_msm_g2(bases, scalars).map_err(|_| 0)
    }

    /// Projective multiplication jumping into the user-defined `mul_projective_g2` hook.
    ///
    /// On any *external* error returns `Projective::zero()`.
    #[inline(always)]
    fn mul_projective(base: &G2Projective<H>, scalar: &[u64]) -> G2Projective<H> {
        H::bn254_mul_projective_g2(base, scalar).unwrap_or_default()
    }

    /// Affine multiplication jumping into the user-defined `mul_projective_g2` hook.
    ///
    /// On any *external* error returns `Projective::zero()`.
    #[inline(always)]
    fn mul_affine(base: &G2Affine<H>, scalar: &[u64]) -> G2Projective<H> {
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
    fn is_in_correct_subgroup_assuming_on_curve(point: &G2Affine<H>) -> bool {
        // Subgroup check from section 4.3 of https://eprint.iacr.org/2022/352.pdf.
        //
        // Checks that [p]P = [6X^2]P

        let x_times_point = point.mul_bigint(SIX_X_SQUARED);
        let p_times_point = p_power_endomorphism(point);
        x_times_point.eq(&p_times_point)
    }
}

/// psi(P) is the untwist-Frobenius-twist endomorphism on E'(Fq2)
fn p_power_endomorphism<H: CurveHooks>(p: &G2Affine<H>) -> G2Affine<H> {
    // Maps (x,y) -> (x^p * (u+9)^((p-1)/3), y^p * (u+9)^((p-1)/2))

    let mut res = *p;
    res.x.frobenius_map_in_place(1);
    res.y.frobenius_map_in_place(1);

    res.x *= P_POWER_ENDOMORPHISM_COEFF_0;
    res.y *= P_POWER_ENDOMORPHISM_COEFF_1;

    res
}
