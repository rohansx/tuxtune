use toml::Value;

const DEVELOPER_TOML: &str = include_str!("../profiles/developer.toml");
const COMPILE_TOML: &str = include_str!("../profiles/compile.toml");
const GAMING_TOML: &str = include_str!("../profiles/gaming.toml");
const BATTERY_TOML: &str = include_str!("../profiles/battery.toml");
const SERVER_TOML: &str = include_str!("../profiles/server.toml");

fn all_profiles() -> Vec<(&'static str, &'static str)> {
    vec![
        ("developer", DEVELOPER_TOML),
        ("compile", COMPILE_TOML),
        ("gaming", GAMING_TOML),
        ("battery", BATTERY_TOML),
        ("server", SERVER_TOML),
    ]
}

#[test]
fn all_profiles_parse() {
    for (name, toml_str) in all_profiles() {
        let val: Value = toml::from_str(toml_str)
            .unwrap_or_else(|e| panic!("Failed to parse {name}.toml: {e}"));
        assert!(val.is_table(), "{name}.toml root should be a table");
    }
}

#[test]
fn all_profiles_have_metadata() {
    for (name, toml_str) in all_profiles() {
        let val: Value = toml::from_str(toml_str).unwrap();
        let profile = val
            .get("profile")
            .unwrap_or_else(|| panic!("{name}.toml missing [profile] table"));
        assert!(
            profile.get("name").and_then(|v| v.as_str()).is_some(),
            "{name}.toml missing profile.name"
        );
        assert!(
            profile
                .get("description")
                .and_then(|v| v.as_str())
                .is_some(),
            "{name}.toml missing profile.description"
        );
    }
}

#[test]
fn all_profiles_have_optimizations_or_extends() {
    for (name, toml_str) in all_profiles() {
        let val: Value = toml::from_str(toml_str).unwrap();
        let has_opts = val
            .get("optimization")
            .and_then(|v| v.as_array())
            .is_some_and(|a| !a.is_empty());
        let has_extends = val
            .get("profile")
            .and_then(|p| p.get("extends"))
            .and_then(|v| v.as_array())
            .is_some_and(|a| !a.is_empty());
        assert!(
            has_opts || has_extends,
            "{name}.toml has neither optimizations nor extends"
        );
    }
}

#[test]
fn developer_profile_has_expected_optimizations() {
    let val: Value = toml::from_str(DEVELOPER_TOML).unwrap();
    let opts = val["optimization"].as_array().unwrap();
    let names: Vec<&str> = opts
        .iter()
        .map(|o| o["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"Inotify watches"));
    assert!(names.contains(&"TCP buffer sizes"));
    assert!(names.contains(&"TCP Fast Open"));
    assert!(names.contains(&"Connection handling"));
}

#[test]
fn compile_extends_developer() {
    let val: Value = toml::from_str(COMPILE_TOML).unwrap();
    let extends = val["profile"]["extends"].as_array().unwrap();
    let parents: Vec<&str> = extends.iter().map(|v| v.as_str().unwrap()).collect();
    assert!(parents.contains(&"developer"));
}

#[test]
fn server_extends_developer() {
    let val: Value = toml::from_str(SERVER_TOML).unwrap();
    let extends = val["profile"]["extends"].as_array().unwrap();
    let parents: Vec<&str> = extends.iter().map(|v| v.as_str().unwrap()).collect();
    assert!(parents.contains(&"developer"));
}
