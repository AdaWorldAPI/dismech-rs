//! The 512-byte SoA `NodeRow` byte layout -- a **byte-layout contract, not a
//! code dependency**, mirroring `ogar-obo`'s own posture exactly (that
//! crate depends on nothing from `lance-graph-contract` either; see its
//! `lib.rs` doc comment "The loader connection is a BYTE-LAYOUT contract").
//! `lance-graph`'s `node_rows_from_le_bytes` reads these bytes back as
//! `&[NodeRow]` with no deserialize.
//!
//! ```text
//!   key (16, LE)                            edges (16)         value (480)
//!   ┌────────┬────┬────┬────┬──────┬──────┐ ┌──────────────┐  ┌─────────────┐
//!   classid  HEEL HIP  TWIG family identity 1 byte/predicate  tenant slab
//!   u32(4)   u16  u16  u16  u16(2) u16(2)   slot               (name len, ...)
//!   └────────┴────┴────┴────┴──────┴──────┘ └──────────────┘  └─────────────┘
//! ```
//!
//! **Mirrored addressing (the "same node, different classid" rule):** where
//! a disorder carries a resolvable `MONDO:<num>` xref, its DisMech-domain
//! `identity` is set to that SAME MONDO numeric -- not a fresh ordinal. Two
//! rows then differ ONLY in `classid` (`0x0301_xxxx` MONDO vs
//! `0x0333_xxxx` DisMech), which is what makes a cross-domain join a
//! position comparison instead of a table lookup: `unpack_key` on either
//! row yields the identical `identity`.

/// Row stride of the canonical SoA node -- `key(16) + edges(16) + value(480)`.
pub const NODE_ROW_STRIDE: usize = 512;
/// Byte offset of the edge block within a row.
pub const EDGES_OFFSET: usize = 16;
/// Byte offset of the value slab within a row.
pub const VALUE_OFFSET: usize = 32;

/// DisMech's reserved concept slot in the shared `0x03` Ontology domain --
/// the next free slot after RO's relation-body concept (`0x0306`); measured
/// unused anywhere in `AdaWorldAPI/OGAR` on 2026-08-17 (`grep -rn "0x0333"`,
/// zero hits). `classid = (DISMECH_CONCEPT << 16) | app_prefix`, the same
/// canon-high idiom every OGAR-addressed domain uses.
pub const DISMECH_CONCEPT: u16 = 0x0333;

/// A 64-byte-aligned 512-byte row cell -- the alignment
/// `node_rows_from_le_bytes` requires for a genuine zero-copy `&[NodeRow]`
/// view. Mirrors `ogar_obo::Row512` exactly.
#[derive(Clone, Copy)]
#[repr(C, align(64))]
pub struct Row512(pub [u8; NODE_ROW_STRIDE]);

impl Row512 {
    #[must_use]
    pub const fn zeroed() -> Self {
        Row512([0u8; NODE_ROW_STRIDE])
    }
}

/// Build the 16-byte canon key: `classid(4) | HEEL(2) | HIP(2) | TWIG(2) |
/// family(2) | identity(2)`, all little-endian. HEEL/HIP/TWIG stay `0` in
/// this first bake (the zero-fallback ladder: `identity` alone
/// discriminates until an HHTL basin mint wakes routing).
///
/// The CURIE numeric does not fit `identity` alone for large MONDO ids
/// (`MONDO:0700092` = 700_092 > `u16::MAX`), so it rides the V3
/// `family:identity` rail exactly as `ogar_obo::pack_key` does: `family`
/// carries the high 16 bits, `identity` the low 16, recombined by
/// `unpack_key` as `(family << 16) | identity`.
#[must_use]
pub fn pack_key(classid: u32, identity: u32) -> [u8; 16] {
    let mut k = [0u8; 16];
    k[0..4].copy_from_slice(&classid.to_le_bytes());
    let f = ((identity >> 16) as u16).to_le_bytes();
    let i = ((identity & 0xFFFF) as u16).to_le_bytes();
    k[12] = f[0];
    k[13] = f[1];
    k[14] = i[0];
    k[15] = i[1];
    k
}

/// Read a row's key back as `(classid, numeric identity)` -- the inverse of
/// [`pack_key`].
#[must_use]
pub const fn unpack_key(row: &[u8; NODE_ROW_STRIDE]) -> (u32, u32) {
    let classid = u32::from_le_bytes([row[0], row[1], row[2], row[3]]);
    let family = u16::from_le_bytes([row[12], row[13]]) as u32;
    let identity = u16::from_le_bytes([row[14], row[15]]) as u32;
    (classid, (family << 16) | identity)
}

/// Pack one disorder identity node into its 512-byte row.
///
/// Value slab layout (this first slice; additive, positions never reused):
/// - `[0..2)`  `name` byte length (u16 LE) -- `0` means absent
/// - `[2..4)`  `description` byte length (u16 LE) -- `0` means absent
/// - `[4..5)`  category tag: `0`=unknown, `1`=Genetic, `2`=Acquired,
///   `3`=Other (extend additively; never renumber a shipped tag)
/// - `[5..6)`  `1` if this row's identity mirrors a resolved MONDO xref,
///   `0` if it was addressed by fallback ordinal (see [`crate::bake`])
///
/// The name/description TEXT itself is not carried in-row (480 bytes is far
/// too small for free text at scale) -- it rides the out-of-line label
/// lane, mirroring the golden bake's own `obo_full_labels.tsv` pattern.
/// This function returns only the row; the caller pairs it with the label.
#[must_use]
pub fn pack_row(
    classid: u32,
    identity: u32,
    d: &crate::model::DisorderIdentity,
    mondo_mirrored: bool,
) -> Row512 {
    let mut row = Row512::zeroed();
    row.0[0..16].copy_from_slice(&pack_key(classid, identity));
    let name_len = d.name.as_deref().map_or(0, str::len).min(u16::MAX as usize) as u16;
    let desc_len = d
        .description
        .as_deref()
        .map_or(0, str::len)
        .min(u16::MAX as usize) as u16;
    row.0[VALUE_OFFSET..VALUE_OFFSET + 2].copy_from_slice(&name_len.to_le_bytes());
    row.0[VALUE_OFFSET + 2..VALUE_OFFSET + 4].copy_from_slice(&desc_len.to_le_bytes());
    row.0[VALUE_OFFSET + 4] = match d.category.as_deref() {
        Some("Genetic") => 1,
        Some("Acquired") => 2,
        Some(_) => 3,
        None => 0,
    };
    row.0[VALUE_OFFSET + 5] = u8::from(mondo_mirrored);
    row
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::DisorderIdentity;

    /// The round-trip a bake's own reader depends on: what was packed is
    /// what unpacks, for an identity well past `u16::MAX` (the real-world
    /// case `pack_key`'s doc comment names).
    #[test]
    fn pack_key_round_trips_a_large_mondo_numeric() {
        let classid = (u32::from(DISMECH_CONCEPT) << 16) | 0x1000;
        let big_identity = 700_092u32; // > u16::MAX, a real MONDO magnitude
        let k = pack_key(classid, big_identity);
        let mut row = [0u8; NODE_ROW_STRIDE];
        row[0..16].copy_from_slice(&k);
        assert_eq!(unpack_key(&row), (classid, big_identity));
    }

    /// Anti-vacuity: two different identities must NOT unpack to the same
    /// value -- a round-trip test that only checked equality-with-itself
    /// would pass even if `pack_key`/`unpack_key` both had the same bug.
    #[test]
    fn distinct_identities_pack_to_distinct_keys() {
        let classid = u32::from(DISMECH_CONCEPT) << 16;
        let mut a = [0u8; NODE_ROW_STRIDE];
        a[0..16].copy_from_slice(&pack_key(classid, 18_923));
        let mut b = [0u8; NODE_ROW_STRIDE];
        b[0..16].copy_from_slice(&pack_key(classid, 18_924));
        assert_ne!(unpack_key(&a), unpack_key(&b));
    }

    /// The mirrored-addressing claim: a DisMech row and a MONDO row for the
    /// same disease share `identity`, differ only in `classid` -- proving a
    /// cross-domain join is a position comparison, not a lookup.
    #[test]
    fn mirrored_identity_matches_across_domains_differing_only_in_classid() {
        const MONDO_CONCEPT: u16 = 0x0301;
        let app_prefix = 0x2000u32;
        let mondo_id = 18_923u32; // MONDO:0018923, the real corpus sample
        let mondo_classid = (u32::from(MONDO_CONCEPT) << 16) | app_prefix;
        let dismech_classid = (u32::from(DISMECH_CONCEPT) << 16) | app_prefix;
        let mut mondo_row = [0u8; NODE_ROW_STRIDE];
        mondo_row[0..16].copy_from_slice(&pack_key(mondo_classid, mondo_id));
        let mut dismech_row = [0u8; NODE_ROW_STRIDE];
        dismech_row[0..16].copy_from_slice(&pack_key(dismech_classid, mondo_id));
        let (mc, mi) = unpack_key(&mondo_row);
        let (dc, di) = unpack_key(&dismech_row);
        assert_eq!(mi, di, "identity must mirror exactly");
        assert_ne!(mc, dc, "classid must differ -- these are different domains");
    }

    /// Field isolation: writing the value slab must not perturb the key,
    /// and vice versa -- the matrix this workspace's own iron rule
    /// (I-LEGACY-API-FEATURE-GATED) requires for any layout-touching change.
    #[test]
    fn packing_the_value_slab_does_not_disturb_the_key() {
        let classid = u32::from(DISMECH_CONCEPT) << 16;
        let d = DisorderIdentity {
            name: Some("x".repeat(300)),
            description: Some("y".repeat(9000)),
            category: Some("Genetic".to_string()),
            disease_term: None,
        };
        let row = pack_row(classid, 42, &d, true);
        let (c, i) = unpack_key(&row.0);
        assert_eq!((c, i), (classid, 42));
    }

    /// Anti-vacuity for `pack_row`'s category tag: an unrecognized category
    /// string must map to `3` (Other), not silently collapse to `0`
    /// (unknown/absent) -- those are different facts.
    #[test]
    fn an_unrecognized_category_is_other_not_unknown() {
        let d = DisorderIdentity {
            category: Some("Behavioral".to_string()),
            ..Default::default()
        };
        let row = pack_row(0, 1, &d, false);
        assert_eq!(row.0[VALUE_OFFSET + 4], 3);
        let d_absent = DisorderIdentity::default();
        let row_absent = pack_row(0, 1, &d_absent, false);
        assert_eq!(row_absent.0[VALUE_OFFSET + 4], 0);
    }
}
