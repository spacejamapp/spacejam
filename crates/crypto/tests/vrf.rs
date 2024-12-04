use spacejam_crypto::{
    ring::{RING_CTX, RING_SIZE},
    vrf::*,
};

macro_rules! measure_time {
    ($func_name:expr, $func_call:expr) => {{
        let start = std::time::Instant::now();
        let result = $func_call;
        let duration = start.elapsed();
        println!("* Time taken by {}: {:?}", $func_name, duration);
        result
    }};
}
#[test]
fn test_vrf() -> anyhow::Result<()> {
    let mut ring: Vec<_> = (0..RING_SIZE)
        .map(|i| Secret::from_seed(&i.to_le_bytes()).public())
        .collect();
    let prover_key_index = 3;

    // NOTE: any key can be replaced with the padding point
    let padding_point = Public::from(RING_CTX.padding_point());
    ring[2] = padding_point;
    ring[5] = padding_point;

    let prover = Prover::new(ring.clone(), prover_key_index);
    let verifier = Verifier::new(ring);

    let vrf_input_data = b"foo";

    //--- Anonymous VRF

    let aux_data = b"bar";

    // Prover signs some data.
    let ring_signature = measure_time! {
        "ring-vrf-sign",
        prover.ring_vrf_sign(vrf_input_data, aux_data)
    }?;

    // Verifier checks it without knowing who is the signer.
    let ring_vrf_output = measure_time! {
        "ring-vrf-verify",
        verifier.ring_vrf_verify(vrf_input_data, aux_data, &ring_signature).unwrap()
    };

    //--- Non anonymous VRF

    let other_aux_data = b"hello";

    // Prover signs the same vrf-input data (we want the output to match)
    // But different aux data.
    let ietf_signature = measure_time! {
        "ietf-vrf-sign",
        prover.ietf_vrf_sign(vrf_input_data, other_aux_data)
    }?;

    // Verifier checks the signature knowing the signer identity.
    let ietf_vrf_output = measure_time! {
        "ietf-vrf-verify",
        verifier.ietf_vrf_verify(vrf_input_data, other_aux_data, &ietf_signature, prover_key_index).unwrap()
    };

    // Must match
    assert_eq!(ring_vrf_output, ietf_vrf_output);
    Ok(())
}
