use squire_core::table::{FieldKind, Table};

fn por() -> Table {
    Table::pool_of_radiance()
}

#[test]
fn the_builtin_pool_of_radiance_table_loads() {
    let t = por();

    assert_eq!(t.game, "Pool of Radiance");
    assert_eq!(
        t.record_len, 285,
        "the record is 285 bytes, offsets 0x000..=0x11C"
    );
}

#[test]
fn describes_the_fields_v1_reads() {
    let t = por();

    // Offsets are from GBC's own table, docs/gbc/Resources/Character file
    // formats/01. Pool of Radiance.txt. They are the independent source of
    // truth, not something this crate computes.
    assert_eq!(t.field("name").unwrap().offset, 0x000);
    assert_eq!(t.field("hit_points_current").unwrap().offset, 0x11B);
    assert_eq!(t.field("hit_points_maximum").unwrap().offset, 0x032);
    assert_eq!(t.field("class").unwrap().offset, 0x02F);
    assert_eq!(t.field("race").unwrap().offset, 0x02E);
    assert_eq!(t.field("status").unwrap().offset, 0x10C);
}

#[test]
fn the_name_field_is_a_length_prefixed_string_of_fifteen_bytes() {
    let t = por();
    let name = t.field("name").unwrap();

    assert_eq!(name.kind, FieldKind::PascalString);
    assert_eq!(name.len, 16, "one length byte plus fifteen characters");
}

#[test]
fn enumerations_carry_their_value_names() {
    let t = por();

    assert_eq!(t.enum_name("class", 0x03), Some("paladin"));
    assert_eq!(t.enum_name("class", 0x04), Some("ranger"));
    assert_eq!(t.enum_name("race", 0x07), Some("human"));
    assert_eq!(t.enum_name("status", 0x06), Some("dead"));
    assert_eq!(
        t.enum_name("class", 0xFE),
        None,
        "an unknown value has no name"
    );
}

#[test]
fn every_field_fits_inside_the_record() {
    let t = por();

    for f in &t.fields {
        assert!(
            f.offset + f.len <= t.record_len,
            "field {} runs past the end of the record",
            f.name
        );
    }
}

#[test]
fn no_two_fields_overlap() {
    let t = por();
    let mut sorted = t.fields.clone();
    sorted.sort_by_key(|f| f.offset);

    for pair in sorted.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        assert!(
            a.offset + a.len <= b.offset,
            "field {} overlaps field {}",
            a.name,
            b.name
        );
    }
}

#[test]
fn a_field_that_runs_past_the_record_is_rejected_at_load() {
    let bad = r#"
game = "Broken"
record_len = 8

[[field]]
name = "way_out_there"
offset = 6
len = 4
kind = "u32le"
"#;

    let err = Table::from_toml(bad).unwrap_err();

    assert!(
        err.to_string().contains("way_out_there"),
        "the error names the offending field, got: {err}"
    );
}

#[test]
fn a_duplicate_field_name_is_rejected_at_load() {
    let bad = r#"
game = "Broken"
record_len = 8

[[field]]
name = "hp"
offset = 0
len = 1
kind = "u8"

[[field]]
name = "hp"
offset = 1
len = 1
kind = "u8"
"#;

    let err = Table::from_toml(bad).unwrap_err();

    assert!(err.to_string().contains("hp"), "got: {err}");
}

#[test]
fn a_width_that_contradicts_the_kind_is_rejected_at_load() {
    let bad = r#"
game = "Broken"
record_len = 8

[[field]]
name = "not_really_a_word"
offset = 0
len = 3
kind = "u16le"
"#;

    let err = Table::from_toml(bad).unwrap_err();

    assert!(err.to_string().contains("not_really_a_word"), "got: {err}");
}

#[test]
fn the_armor_class_fields_carry_the_sixty_minus_transform() {
    let t = por();

    use squire_core::table::Transform;
    assert_eq!(
        t.field("armor_class_current").unwrap().transform,
        Some(Transform::SixtyMinus)
    );
    assert_eq!(
        t.field("thac0_current").unwrap().transform,
        Some(Transform::SixtyMinus)
    );
    assert_eq!(t.field("hit_points_current").unwrap().transform, None);
}

#[test]
fn a_transform_on_a_multi_byte_field_is_rejected_at_load() {
    let bad = r#"
game = "Broken"
record_len = 8

[[field]]
name = "two_wide"
offset = 0
len = 2
kind = "u16le"
transform = "sixty_minus"
"#;

    let err = Table::from_toml(bad).unwrap_err();

    assert!(err.to_string().contains("two_wide"), "got: {err}");
}
