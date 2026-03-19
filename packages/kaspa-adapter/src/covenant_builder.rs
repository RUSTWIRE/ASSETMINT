// DISCLAIMER: Technical demo code — legal wrappers required in production
// SPDX-License-Identifier: MIT
//
//! Hand-built covenant scripts using the KTT-proven pattern.
//! These bypass silverc's witness encoding issues by constructing
//! raw opcode sequences directly.
//!
//! The KTT (Kaspa Trust Tag) approach constructs P2SH redeem scripts
//! from raw opcodes, proven on TN12 via TX 27385b04. The pattern:
//!   Redeem script: [push_32][pubkey] OP_CHECKSIG
//!   ScriptSig:     [sig_push] [redeem_script_push]

/// Kaspa script opcodes (from KTT's verified opcode list)
pub mod op {
    pub const FALSE: u8 = 0x00;
    pub const TRUE: u8 = 0x51;
    pub const DROP: u8 = 0x75;
    pub const VERIFY: u8 = 0x69;
    pub const EQUAL: u8 = 0x87;
    pub const CHECKSIG: u8 = 0xac;
    pub const BLAKE2B: u8 = 0xaa;
    // KIP-10 introspection
    pub const INPUTINDEX: u8 = 0xb9;
    pub const INPUTVALUE: u8 = 0xbe;   // was 0xc4
    pub const INPUTSCRIPT: u8 = 0xbf;  // was 0xc5
    pub const OUTPUTVALUE: u8 = 0xc2;
    pub const OUTPUTSCRIPT: u8 = 0xc3;
    pub const LESSTHANOREQUAL: u8 = 0xa1;
}

/// Build a simple CHECKSIG covenant (proven on TN12)
///
/// Redeem script: `[push_32] [pubkey] OP_CHECKSIG`
///
/// This is equivalent to P2PK but wrapped in P2SH. Proven working
/// on Kaspa TN12 by TX 27385b04.
pub fn build_checksig_covenant(owner_pubkey: &[u8; 32]) -> Vec<u8> {
    let mut script = Vec::new();
    script.push(0x20); // push 32 bytes
    script.extend_from_slice(owner_pubkey);
    script.push(op::CHECKSIG);
    script
}

/// Build a compliance-gated transfer covenant (KTT-style)
///
/// Verifies owner signature AND enforces value conservation via KIP-10
/// introspection opcodes. The covenant ensures that output[0].value
/// does not exceed the input value being spent (no inflation).
///
/// Script logic:
/// ```text
/// [push_32] [pubkey] OP_CHECKSIG OP_VERIFY     -- sig check
/// OP_0 OP_OUTPUTVALUE                           -- push output[0].value
/// OP_INPUTINDEX OP_INPUTVALUE                   -- push current input value
/// OP_LESSTHANOREQUAL OP_VERIFY                  -- out <= in
/// OP_TRUE                                       -- success
/// ```
pub fn build_compliance_covenant(owner_pubkey: &[u8; 32]) -> Vec<u8> {
    let mut script = Vec::new();
    // Push owner pubkey, CHECKSIG, VERIFY
    script.push(0x20);
    script.extend_from_slice(owner_pubkey);
    script.push(op::CHECKSIG);
    script.push(op::VERIFY);
    // KIP-10: output[0].value <= input.value (value conservation)
    script.push(0x00); // push 0 (output index) — OP_FALSE doubles as push-0
    script.push(op::OUTPUTVALUE);
    script.push(op::INPUTINDEX);
    script.push(op::INPUTVALUE);
    script.push(op::LESSTHANOREQUAL);
    script.push(op::VERIFY);
    // Leave TRUE on stack
    script.push(op::TRUE);
    script
}

/// Build a self-propagating covenant (KTT Trust Tag pattern)
///
/// Signature verified + output must carry same covenant script.
/// This enforces that the covenant cannot be "stripped" — the output
/// must be locked by the same P2SH script, preserving the covenant
/// across transfers.
///
/// Script logic:
/// ```text
/// [push_32] [pubkey] OP_CHECKSIG OP_VERIFY         -- sig check
/// OP_0 OP_OUTPUTSCRIPT OP_INPUTINDEX OP_INPUTSCRIPT -- compare scripts
/// OP_EQUAL OP_VERIFY                                -- must match
/// OP_0 OP_OUTPUTVALUE OP_INPUTINDEX OP_INPUTVALUE   -- compare values
/// OP_LESSTHANOREQUAL OP_VERIFY                      -- out <= in
/// OP_TRUE                                           -- success
/// ```
pub fn build_self_propagating_covenant(owner_pubkey: &[u8; 32]) -> Vec<u8> {
    let mut script = Vec::new();
    // CHECKSIG VERIFY
    script.push(0x20);
    script.extend_from_slice(owner_pubkey);
    script.push(op::CHECKSIG);
    script.push(op::VERIFY);
    // Self-propagation: output[0].script == input.script
    script.push(0x00);
    script.push(op::OUTPUTSCRIPT);
    script.push(op::INPUTINDEX);
    script.push(op::INPUTSCRIPT);
    script.push(op::EQUAL);
    script.push(op::VERIFY);
    // Value conservation
    script.push(0x00);
    script.push(op::OUTPUTVALUE);
    script.push(op::INPUTINDEX);
    script.push(op::INPUTVALUE);
    script.push(op::LESSTHANOREQUAL);
    script.push(op::VERIFY);
    script.push(op::TRUE);
    script
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksig_covenant_structure() {
        let pubkey = [0xab_u8; 32];
        let script = build_checksig_covenant(&pubkey);
        // Expected: 0x20 + 32 bytes pubkey + 0xac = 34 bytes
        assert_eq!(script.len(), 34);
        assert_eq!(script[0], 0x20);           // push 32
        assert_eq!(&script[1..33], &pubkey);   // pubkey
        assert_eq!(script[33], op::CHECKSIG);  // OP_CHECKSIG
    }

    #[test]
    fn test_compliance_covenant_structure() {
        let pubkey = [0xcd_u8; 32];
        let script = build_compliance_covenant(&pubkey);
        // push32 + 32 + CHECKSIG + VERIFY + 0x00 + OUTPUTVALUE + INPUTINDEX + INPUTVALUE + LTE + VERIFY + TRUE
        // = 1 + 32 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 = 42
        assert_eq!(script.len(), 42);
        // Verify the CHECKSIG+VERIFY prefix
        assert_eq!(script[0], 0x20);
        assert_eq!(script[33], op::CHECKSIG);
        assert_eq!(script[34], op::VERIFY);
        // Verify the KIP-10 introspection suffix
        assert_eq!(script[35], 0x00);                   // output index 0
        assert_eq!(script[36], op::OUTPUTVALUE);
        assert_eq!(script[37], op::INPUTINDEX);
        assert_eq!(script[38], op::INPUTVALUE);
        assert_eq!(script[39], op::LESSTHANOREQUAL);
        assert_eq!(script[40], op::VERIFY);
        assert_eq!(script[41], op::TRUE);
    }

    #[test]
    fn test_self_propagating_covenant_structure() {
        let pubkey = [0xef_u8; 32];
        let script = build_self_propagating_covenant(&pubkey);
        // push32 + 32 + CHECKSIG + VERIFY
        // + 0x00 + OUTPUTSCRIPT + INPUTINDEX + INPUTSCRIPT + EQUAL + VERIFY
        // + 0x00 + OUTPUTVALUE + INPUTINDEX + INPUTVALUE + LTE + VERIFY + TRUE
        // 1(push32) + 32(pubkey) + 15(opcodes) = 48
        assert_eq!(script.len(), 48);
        // Check self-propagation section starts after CHECKSIG VERIFY
        assert_eq!(script[35], 0x00);                   // output index 0
        assert_eq!(script[36], op::OUTPUTSCRIPT);
        assert_eq!(script[37], op::INPUTINDEX);
        assert_eq!(script[38], op::INPUTSCRIPT);
        assert_eq!(script[39], op::EQUAL);
        assert_eq!(script[40], op::VERIFY);
        // Check value conservation section
        assert_eq!(script[41], 0x00);
        assert_eq!(script[42], op::OUTPUTVALUE);
        // Final TRUE
        assert_eq!(script[47], op::TRUE);
    }

    #[test]
    fn test_different_pubkeys_produce_different_scripts() {
        let pk_a = [0x01_u8; 32];
        let pk_b = [0x02_u8; 32];
        let script_a = build_checksig_covenant(&pk_a);
        let script_b = build_checksig_covenant(&pk_b);
        assert_ne!(script_a, script_b);
    }
}
