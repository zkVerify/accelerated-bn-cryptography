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

use crate::models::{
    bn::BnConfig,
    short_weierstrass::{Affine, Projective},
};
use ark_ec::{AffineRepr, CurveGroup};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::vec::Vec;
use core::fmt;

pub type G1Affine<P> = Affine<<P as BnConfig>::G1Config>;
pub type G1Projective<P> = Projective<<P as BnConfig>::G1Config>;

#[derive(CanonicalSerialize, CanonicalDeserialize)]
pub struct G1Prepared<P: BnConfig>(pub G1Affine<P>);

impl<P: BnConfig> Copy for G1Prepared<P> {}

impl<P: BnConfig> Clone for G1Prepared<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P: BnConfig> PartialEq for G1Prepared<P> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<P: BnConfig> Eq for G1Prepared<P> {}

impl<P: BnConfig> fmt::Debug for G1Prepared<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("G1Prepared").field(&self.0).finish()
    }
}

impl<P: BnConfig> From<G1Affine<P>> for G1Prepared<P> {
    fn from(other: G1Affine<P>) -> Self {
        G1Prepared(other)
    }
}

impl<P: BnConfig> From<G1Projective<P>> for G1Prepared<P> {
    fn from(q: G1Projective<P>) -> Self {
        q.into_affine().into()
    }
}

impl<'a, P: BnConfig> From<&'a G1Affine<P>> for G1Prepared<P> {
    fn from(other: &'a G1Affine<P>) -> Self {
        G1Prepared(*other)
    }
}

impl<'a, P: BnConfig> From<&'a G1Projective<P>> for G1Prepared<P> {
    fn from(q: &'a G1Projective<P>) -> Self {
        q.into_affine().into()
    }
}

impl<P: BnConfig> G1Prepared<P> {
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl<P: BnConfig> Default for G1Prepared<P> {
    fn default() -> Self {
        G1Prepared(G1Affine::<P>::generator())
    }
}
