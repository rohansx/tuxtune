use toml::Value;

const ALL_PROFILES: &[(&str, &str)] = &[
    ("developer", include_str!("../profiles/developer.toml")),
    ("compile", include_str!("../profiles/compile.toml")),
    ("gaming", include_str!("../profiles/gaming.toml")),
    ("battery", include_str!("../profiles/battery.toml")),
    ("server", include_str!("../profiles/server.toml")),
];

#[test]
fn optimization_entries_have_required_fields() {
    for (name, toml_str) in ALL_PROFILES {
        let val: Value = toml::from_str(toml_str).unwrap();
        if let Some(opts) = val.get("optimization").and_then(|v| v.as_array()) {
            for (i, opt) in opts.iter().enumerate() {
                assert!(
                    opt.get("name").and_then(|v| v.as_str()).is_some(),
                    "{name}.toml optimization[{i}] missing name"
                );
                assert!(
                    opt.get("description").and_then(|v| v.as_str()).is_some(),
                    "{name}.toml optimization[{i}] missing description"
                );
                assert!(
                    opt.get("explanation").and_then(|v| v.as_str()).is_some(),
                    "{name}.toml optimization[{i}] missing explanation"
                );
            }
        }
    }
}

#[test]
fn sysctl_entries_have_key_and_value() {
    for (name, toml_str) in ALL_PROFILES {
        let val: Value = toml::from_str(toml_str).unwrap();
        if let Some(opts) = val.get("optimization").and_then(|v| v.as_array()) {
            for (i, opt) in opts.iter().enumerate() {
                if let Some(sysctls) = opt.get("sysctl").and_then(|v| v.as_array()) {
                    for (j, sysctl) in sysctls.iter().enumerate() {
                        assert!(
                            sysctl.get("key").and_then(|v| v.as_str()).is_some(),
                            "{name}.toml optimization[{i}].sysctl[{j}] missing key"
                        );
                        assert!(
                            sysctl.get("value").and_then(|v| v.as_str()).is_some(),
                            "{name}.toml optimization[{i}].sysctl[{j}] missing value"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn extends_references_are_valid() {
    let known_names: Vec<&str> = ALL_PROFILES.iter().map(|(name, _)| *name).collect();

    for (name, toml_str) in ALL_PROFILES {
        let val: Value = toml::from_str(toml_str).unwrap();
        if let Some(extends) = val
            .get("profile")
            .and_then(|p| p.get("extends"))
            .and_then(|v| v.as_array())
        {
            for parent in extends {
                let parent_name = parent.as_str().unwrap();
                assert!(
                    known_names.contains(&parent_name),
                    "{name}.toml extends unknown profile: {parent_name}"
                );
                assert_ne!(
                    parent_name, *name,
                    "{name}.toml extends itself"
                );
            }
        }
    }
}

#[test]
fn requires_field_is_string_array() {
    for (name, toml_str) in ALL_PROFILES {
        let val: Value = toml::from_str(toml_str).unwrap();
        if let Some(opts) = val.get("optimization").and_then(|v| v.as_array()) {
            for (i, opt) in opts.iter().enumerate() {
                if let Some(requires) = opt.get("requires") {
                    let arr = requires.as_array().unwrap_or_else(|| {
                        panic!("{name}.toml optimization[{i}].requires is not an array")
                    });
                    for (j, req) in arr.iter().enumerate() {
                        assert!(
                            req.as_str().is_some(),
                            "{name}.toml optimization[{i}].requires[{j}] is not a string"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn min_ram_mb_is_positive_integer() {
    for (name, toml_str) in ALL_PROFILES {
        let val: Value = toml::from_str(toml_str).unwrap();
        if let Some(opts) = val.get("optimization").and_then(|v| v.as_array()) {
            for (i, opt) in opts.iter().enumerate() {
                if let Some(min_ram) = opt.get("min_ram_mb") {
                    let v = min_ram.as_integer().unwrap_or_else(|| {
                        panic!("{name}.toml optimization[{i}].min_ram_mb is not an integer")
                    });
                    assert!(
                        v > 0,
                        "{name}.toml optimization[{i}].min_ram_mb must be positive"
                    );
                }
            }
        }
    }
}

#[test]
fn profile_names_match_filenames() {
    for (filename, toml_str) in ALL_PROFILES {
        let val: Value = toml::from_str(toml_str).unwrap();
        let profile_name = val["profile"]["name"].as_str().unwrap();
        assert_eq!(
            profile_name, *filename,
            "profile.name '{profile_name}' doesn't match filename '{filename}.toml'"
        );
    }
}
