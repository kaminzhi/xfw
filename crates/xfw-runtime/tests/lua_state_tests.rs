use mlua::Lua;
use std::path::PathBuf;
use tempfile::TempDir;

fn setup_lua_with_framework() -> (Lua, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let framework_dir = temp_dir.path().join("framework");
    std::fs::create_dir_all(&framework_dir).unwrap();

    let source_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("lua")
        .join("framework");

    std::fs::copy(
        source_dir.join("runtime.lua"),
        framework_dir.join("runtime.lua"),
    )
    .unwrap();

    let lua = Lua::new();

    let lua_dir_str = temp_dir.path().to_string_lossy().to_string();
    let setup_code = format!(
        r#"
        local dir = '{}'
        package.path = package.path .. ';' .. dir .. '/?.lua;' .. dir .. '/?/init.lua'
        "#,
        lua_dir_str.replace('\\', "\\\\").replace('\'', "\\\'")
    );
    lua.load(&setup_code).exec().unwrap();

    lua.globals()
        .set("xfw_state", lua.create_table().unwrap())
        .unwrap();

    (lua, temp_dir)
}

#[test]
fn test_state_creation() {
    let (lua, _temp) = setup_lua_with_framework();

    let code = r#"
        local runtime = require("framework.runtime")
        local store = runtime.UI.state({
            value = 42,
            name = "test"
        })
        return store.value, store.name
    "#;

    let result: (i64, String) = lua.load(code).eval().unwrap();

    assert_eq!(result.0, 42);
    assert_eq!(result.1, "test");
}

#[test]
fn test_state_nested_tables() {
    let (lua, _temp) = setup_lua_with_framework();

    let code = r#"
        local runtime = require("framework.runtime")
        local store = runtime.UI.state({
            nested = {
                inner = "hello"
            }
        })
        return store.nested.inner
    "#;

    let result: String = lua.load(code).eval().unwrap();

    assert_eq!(result, "hello");
}

#[test]
fn test_state_dirty_tracking() {
    let (lua, _temp) = setup_lua_with_framework();

    let code = r#"
        local runtime = require("framework.runtime")
        local store = runtime.UI.state({
            value = 10
        })
        
        store.value = 20
        
        local dirty = runtime.UI.get_dirty()
        return dirty[1].key, dirty[1].value
    "#;

    let result: (String, i64) = lua.load(code).eval().unwrap();

    assert_eq!(result.0, "global.value");
    assert_eq!(result.1, 20);
}

#[test]
fn test_state_nested_dirty_tracking() {
    let (lua, _temp) = setup_lua_with_framework();

    let code = r#"
        local runtime = require("framework.runtime")
        local store = runtime.UI.state({
            battery = {
                level = 100,
                is_charging = false
            }
        })
        
        store.battery.level = 50
        
        local dirty = runtime.UI.get_dirty()
        return dirty[1].key, dirty[1].value
    "#;

    let result: (String, i64) = lua.load(code).eval().unwrap();

    assert_eq!(result.0, "global.battery.level");
    assert_eq!(result.1, 50);
}

#[test]
fn test_state_pairs_iteration() {
    let (lua, _temp) = setup_lua_with_framework();

    let code = r#"
        local runtime = require("framework.runtime")
        
        local store = runtime.UI.state({
            a = 1,
            b = 2,
            c = 3
        })
        
        -- Use the iterator from __pairs directly
        local mt = getmetatable(store)
        local pf = mt.__pairs
        local iter, state, init = pf(store)
        
        local result = {}
        local k = init
        repeat
            k, v = iter(state, k)
            if k ~= nil then
                table.insert(result, tostring(k) .. "=" .. tostring(v))
            end
        until k == nil
        
        return table.concat(result, ",")
    "#;

    let result: String = lua.load(code).eval().unwrap();

    assert!(result.contains("a="), "pairs iteration failed: {}", result);
}

#[test]
fn test_state_raw_access() {
    let (lua, _temp) = setup_lua_with_framework();

    let code = r#"
        local runtime = require("framework.runtime")
        local store = runtime.UI.state({
            value = 42
        })
        
        local raw = store.__raw
        return raw.value
    "#;

    let result: i64 = lua.load(code).eval().unwrap();

    assert_eq!(result, 42);
}

#[test]
fn test_state_is_reactive() {
    let (lua, _temp) = setup_lua_with_framework();

    let code = r#"
        local runtime = require("framework.runtime")
        local store = runtime.UI.state({
            value = 10
        })
        
        return store.__is_state == true
    "#;

    let result: bool = lua.load(code).eval().unwrap();

    assert!(result);
}

#[test]
fn test_state_path() {
    let (lua, _temp) = setup_lua_with_framework();

    let code = r#"
        local runtime = require("framework.runtime")
        local store = runtime.UI.state({
            nested = {
                deep = "value"
            }
        })
        
        return store.nested.__path
    "#;

    let result: String = lua.load(code).eval().unwrap();

    assert_eq!(result, "global.nested");
}

#[test]
fn test_multiple_state_stores() {
    let (lua, _temp) = setup_lua_with_framework();

    let code = r#"
        local runtime = require("framework.runtime")
        local store1 = runtime.UI.state({ key = "value1" })
        local store2 = runtime.UI.state({ key = "value2" })
        
        store1.key = "changed1"
        store2.key = "changed2"
        
        local dirty = runtime.UI.get_dirty()
        return dirty[1].value, dirty[2].value
    "#;

    let result: (String, String) = lua.load(code).eval().unwrap();

    assert_eq!(result.0, "changed1");
    assert_eq!(result.1, "changed2");
}

#[test]
fn test_state_boolean_values() {
    let (lua, _temp) = setup_lua_with_framework();

    let code = r#"
        local runtime = require("framework.runtime")
        local store = runtime.UI.state({
            enabled = true,
            visible = false
        })
        
        store.enabled = false
        store.visible = true
        
        local dirty = runtime.UI.get_dirty()
        return dirty[1].key, dirty[1].value, dirty[2].key, dirty[2].value
    "#;

    let result: (String, bool, String, bool) = lua.load(code).eval().unwrap();

    assert_eq!(result.0, "global.enabled");
    assert_eq!(result.1, false);
    assert_eq!(result.2, "global.visible");
    assert_eq!(result.3, true);
}

#[test]
fn test_state_clear_dirty() {
    let (lua, _temp) = setup_lua_with_framework();

    let code = r#"
        local runtime = require("framework.runtime")
        local store = runtime.UI.state({ value = 1 })
        
        store.value = 2
        runtime.UI.clear_dirty()
        
        local dirty = runtime.UI.get_dirty()
        return #dirty
    "#;

    let result: usize = lua.load(code).eval().unwrap();

    assert_eq!(result, 0);
}
