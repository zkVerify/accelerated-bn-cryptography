# Accelerated BN Cryptography

## Overview

This library extends [arkworks-rs/algebra](https://github.com/arkworks-rs/algebra).

We fork the popular `BN254` elliptic curve in a way which allows delegating some of the most computationally expensive operations (like pairings, MSM, and final exponentiation) to some user defined hooks.

We also provide a `BN` model to avoid the point preparation before the hooks calls during pairing operations. Therefore, we redefine the elliptic curve sub-groups `G2` for both models as thin wrappers around the affine points and move the point preparation procedure to the user defined hook.

## Usage

The following usage example is modified from the hooks provided by this project.

The project provides a set of ready to use `CurveHooks` implementations compatible with [Substrate](https://github.com/paritytech/polkadot-sdk/primitives/crypto/ec-utils) host functions to jump from *wasm32* computational domain into the native host.

The motivation is:

- native target is typically more efficient that *wasm32*.
- *wasm32* is single thread while in the native target we can hopefully leverage the Arkworks `parallel` feature.

Please note that Substrate elliptic curves host functions take and return raw byte arrays representing SCALE encoded values.

### BN254

```rust
use ark_ec::{
    pairing::{MillerLoopOutput, Pairing},
    short_weierstrass::{Affine as SWAffine, Projective as SWProjective, SWCurveConfig},
    twisted_edwards::{Affine as TEAffine, Projective as TEProjective, TECurveConfig},
    CurveConfig, VariableBaseMSM,
};
use ark_scale::{
    ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate},
    scale::{Decode, Encode},
};
use ark_std::vec::Vec;

use crate::CurveHooks;
use ark_bn254::{g1::Config as ArkG1Config, g2::Config as ArkG2Config, Bn254 as ArkBn254};


const SCALE_USAGE: u8 = ark_scale::make_usage(Compress::No, Validate::No);
type ArkScale<T> = ark_scale::ArkScale<T, SCALE_USAGE>;
type ArkScaleProjective<T> = ark_scale::hazmat::ArkScaleProjective<T>;

#[derive(Copy, Clone)]
pub struct HostHooks;

type Bn254 = ark_bn254_ext::Bn254<HostHooks>;
type G1Affine = ark_bn254_ext::g1::G1Affine<HostHooks>;
type G1Config = ark_bn254_ext::g1::Config<HostHooks>;
type G2Affine = ark_bn254_ext::g2::G2Affine<HostHooks>;
type G2Config = ark_bn254_ext::g2::Config<HostHooks>;

trait TryTransmute {
    fn try_transmute<U: CanonicalDeserialize>(self) -> Result<U, ()>;
}

impl<T: CanonicalSerialize> TryTransmute for T {
    fn try_transmute<U: CanonicalDeserialize>(self) -> Result<U, ()> {
        let buf = ArkScale::from(self).encode();
        ArkScale::<U>::decode(&mut &buf[..])
            .map(|v| v.0)
            .map_err(|_| ())
    }
}

pub fn multi_miller_loop_generic<ExtPairing: Pairing, ArkPairing: Pairing>(
    g1: impl Iterator<Item = ExtPairing::G1Prepared>,
    g2: impl Iterator<Item = ExtPairing::G2Prepared>,
) -> Result<ExtPairing::TargetField, ()> {
    let g1: Vec<ArkPairing::G1Affine> = g1.collect::<Vec<_>>().try_transmute()?;
    let g2: Vec<ArkPairing::G2Affine> = g2.collect::<Vec<_>>().try_transmute()?;

    let res = ArkPairing::multi_miller_loop(g1, g2).0;
    res.try_transmute()
}

pub fn final_exponentiation_generic<ExtPairing: Pairing, ArkPairing: Pairing>(
    target: ExtPairing::TargetField,
) -> Result<ExtPairing::TargetField, ()> {
    let target: ArkPairing::TargetField = target.try_transmute()?;

    let res = ArkPairing::final_exponentiation(MillerLoopOutput(target)).ok_or(())?;
    res.try_transmute()
}

pub fn msm_sw_generic<ExtCurve: SWCurveConfig, ArkCurve: SWCurveConfig>(
    bases: &[SWAffine<ExtCurve>],
    scalars: &[ExtCurve::ScalarField],
) -> Result<SWProjective<ExtCurve>, ()> {
    let bases: Vec<SWAffine<ArkCurve>> = bases.try_transmute()?;
    let scalars: Vec<ArkCurve::ScalarField> = scalars.try_transmute()?;

    let res = <SWProjective<ArkCurve> as VariableBaseMSM>::msm(&bases, &scalars).map_err(|_| ())?;
    res.try_transmute()
}

pub fn mul_projective_sw_generic<ExtConfig: SWCurveConfig, ArkConfig: SWCurveConfig>(
    base: &SWProjective<ExtConfig>,
    scalar: &[u64],
) -> Result<SWProjective<ExtConfig>, ()> {
    let base: SWProjective<ArkConfig> = base.try_transmute()?;

    let res = <ArkConfig as SWCurveConfig>::mul_projective(&base, scalar);
    res.try_transmute()
}

impl CurveHooks for HostHooks {
    fn bn254_multi_miller_loop(
        g1: impl Iterator<Item = <Bn254 as Pairing>::G1Prepared>,
        g2: impl Iterator<Item = <Bn254 as Pairing>::G2Prepared>,
    ) -> Result<<Bn254 as Pairing>::TargetField, ()> {
        multi_miller_loop_generic::<Bn254, ArkBn254>(g1, g2)
    }

    fn bn254_final_exponentiation(
        target: <Bn254 as Pairing>::TargetField,
    ) -> Result<<Bn254 as Pairing>::TargetField, ()> {
        final_exponentiation_generic::<Bn254, ArkBn254>(target)
    }

    fn bn254_msm_g1(
        bases: &[G1Affine],
        scalars: &[<G1Config as CurveConfig>::ScalarField],
    ) -> Result<G1Projective, ()> {
        msm_sw_generic::<G1Config, ArkG1Config>(bases, scalars)
    }

    fn bn254_msm_g2(
        bases: &[G2Affine],
        scalars: &[<G2Config as CurveConfig>::ScalarField],
    ) -> Result<G2Projective, ()> {
        msm_sw_generic::<G2Config, ArkG2Config>(bases, scalars)
    }

    fn bn254_mul_projective_g1(base: &G1Projective, scalar: &[u64]) -> Result<G1Projective, ()> {
        mul_projective_sw_generic::<G1Config, ArkG1Config>(base, scalar)
    }

    fn bn254_mul_projective_g2(base: &G2Projective, scalar: &[u64]) -> Result<G2Projective, ()> {
        mul_projective_sw_generic::<G2Config, ArkG2Config>(base, scalar)
    }
}
```

## ⚠️ Known Limitations ⚠️

Be aware that, while in the hook context, any usage of functions which may re-enter into the same hook with the same value, may cause an infinite loop.

We are aware of hooks re-entrancy issues when using **point checked deserialization** in projective multiplication hooks.

In particular, if you serialize and deserialize (**with point checking**) the input point in one of the projective multiplication hooks then we end up re-entering the multiplication hook with the same value as a consequence of the internally performed check.

The following invocation flow applies:

1. Validation of deserialized value.
2. Check if point is in the correct subgroup.
3. Jump into the `TECurveConfig` for the check.
4. Calls the "custom" (defined by this crate) implementation of `mul_affine` which calls `mul_projective`.
5. Goto 1.

So pay special attention to the actions in your `CurveHooks` implementations.

If you encounter any other way to trigger the open, please file an issue.