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

pub type G2Affine<P> = Affine<<P as BnConfig>::G2Config>;
pub type G2Projective<P> = Projective<<P as BnConfig>::G2Config>;

#[derive(CanonicalSerialize, CanonicalDeserialize)]
pub struct G2Prepared<P: BnConfig>(pub G2Affine<P>);

impl<P: BnConfig> Copy for G2Prepared<P> {}

impl<P: BnConfig> Clone for G2Prepared<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P: BnConfig> PartialEq for G2Prepared<P> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<P: BnConfig> Eq for G2Prepared<P> {}

impl<P: BnConfig> fmt::Debug for G2Prepared<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("G2Prepared").field(&self.0).finish()
    }
}

impl<P: BnConfig> From<G2Affine<P>> for G2Prepared<P> {
    fn from(other: G2Affine<P>) -> Self {
        G2Prepared(other)
    }
}

impl<P: BnConfig> From<G2Projective<P>> for G2Prepared<P> {
    fn from(q: G2Projective<P>) -> Self {
        q.into_affine().into()
    }
}

impl<'a, P: BnConfig> From<&'a G2Affine<P>> for G2Prepared<P> {
    fn from(other: &'a G2Affine<P>) -> Self {
        G2Prepared(*other)
    }
}

impl<'a, P: BnConfig> From<&'a G2Projective<P>> for G2Prepared<P> {
    fn from(q: &'a G2Projective<P>) -> Self {
        q.into_affine().into()
    }
}

impl<P: BnConfig> G2Prepared<P> {
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl<P: BnConfig> Default for G2Prepared<P> {
    fn default() -> Self {
        G2Prepared(G2Affine::<P>::generator())
    }
}
