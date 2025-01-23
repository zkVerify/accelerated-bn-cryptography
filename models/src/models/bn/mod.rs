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

use crate::models::short_weierstrass::SWCurveConfig;
pub use ark_ec::models::bn::TwistType;
use ark_ec::{
    models::CurveConfig,
    pairing::{MillerLoopOutput, Pairing, PairingOutput},
};
use ark_ff::{
    fields::{
        fp12_2over3over2::{Fp12, Fp12Config},
        fp2::Fp2Config,
        fp6_3over2::Fp6Config,
        Fp2,
    },
    PrimeField,
};
use ark_std::marker::PhantomData;
use core::{
    fmt,
    hash::{Hash, Hasher},
};

pub trait BnConfig: 'static + Sized {
    /// Parameterizes the BN family.
    const X: &'static [u64];
    /// Whether or not `X` is negative.
    const X_IS_NEGATIVE: bool;

    /// The absolute value of `6X + 2`.
    const ATE_LOOP_COUNT: &'static [i8];

    /// What kind of twist is this?
    const TWIST_TYPE: TwistType;

    // Field Extension Tower.
    type Fp: PrimeField + Into<<Self::Fp as PrimeField>::BigInt>;
    type Fp2Config: Fp2Config<Fp = Self::Fp>;
    type Fp6Config: Fp6Config<Fp2Config = Self::Fp2Config>;
    type Fp12Config: Fp12Config<Fp6Config = Self::Fp6Config>;
    type G1Config: SWCurveConfig<BaseField = Self::Fp>;
    type G2Config: SWCurveConfig<
        BaseField = Fp2<Self::Fp2Config>,
        ScalarField = <Self::G1Config as CurveConfig>::ScalarField,
    >;

    fn multi_miller_loop(
        a_vec: impl IntoIterator<Item = impl Into<G1Prepared<Self>>>,
        b_vec: impl IntoIterator<Item = impl Into<G2Prepared<Self>>>,
    ) -> MillerLoopOutput<Bn<Self>>;

    fn final_exponentiation(f: MillerLoopOutput<Bn<Self>>) -> Option<PairingOutput<Bn<Self>>>;
}

pub mod g1;
pub mod g2;

pub use self::{
    g1::{G1Affine, G1Prepared, G1Projective},
    g2::{G2Affine, G2Prepared, G2Projective},
};

pub struct Bn<P: BnConfig>(PhantomData<fn() -> P>);

impl<P: BnConfig> Copy for Bn<P> {}

impl<P: BnConfig> Clone for Bn<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P: BnConfig> PartialEq for Bn<P> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<P: BnConfig> Eq for Bn<P> {}

impl<P: BnConfig> fmt::Debug for Bn<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Bn").finish()
    }
}

impl<P: BnConfig> Hash for Bn<P> {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}

impl<P: BnConfig> Pairing for Bn<P> {
    type BaseField = <P::G1Config as CurveConfig>::BaseField;
    type ScalarField = <P::G1Config as CurveConfig>::ScalarField;
    type G1 = G1Projective<P>;
    type G1Affine = G1Affine<P>;
    type G1Prepared = G1Prepared<P>;
    type G2 = G2Projective<P>;
    type G2Affine = G2Affine<P>;
    type G2Prepared = G2Prepared<P>;
    type TargetField = Fp12<P::Fp12Config>;

    fn multi_miller_loop(
        a: impl IntoIterator<Item = impl Into<Self::G1Prepared>>,
        b: impl IntoIterator<Item = impl Into<Self::G2Prepared>>,
    ) -> MillerLoopOutput<Self> {
        P::multi_miller_loop(a, b)
    }

    fn final_exponentiation(f: MillerLoopOutput<Self>) -> Option<PairingOutput<Self>> {
        P::final_exponentiation(f)
    }
}
