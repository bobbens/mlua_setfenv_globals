const ENV: &str = "_ENV"; /// Variable we use for Lua environments

/// Wrapper for Lua environment that can create environments
pub struct Lua {
    /// The true Lua environment
    lua: mlua::Lua,
    /// Our globals, these can be masked but not removed
    globals: mlua::Table,
    /// The metatable for environments, just defaults to our globals
    env_mt: mlua::Table,
}
impl Lua {
    pub fn new() -> mlua::Result<Self> {
        let lua = mlua::Lua::new();
        lua.load_std_libs( mlua::StdLib::ALL_SAFE )?;

        // Swap the global table with an empty one, returning the global one
        let globals: mlua::Table = unsafe { lua.exec_raw( (), |state| {
            mlua::ffi::lua_pushvalue( state, mlua::ffi::LUA_GLOBALSINDEX );
            mlua::ffi::lua_newtable(state);
            mlua::ffi::lua_replace( state, mlua::ffi::LUA_GLOBALSINDEX );
        }) }?;

        // Set globals to be truly read only (should only be possible if the user is being bad)
        let globals_mt = lua.create_table()?;
        globals_mt.set( "__newindex", lua.create_function( |_, ()| -> mlua::Result<()> {
            Err(mlua::Error::RuntimeError(String::from("globals are read only")))
        } )?)?;
        globals.set_metatable( Some(globals_mt) );

        // Our new globals should be usable now
        let wrapped = lua.globals();
        let wrapped_mt = lua.create_table()?;
        wrapped_mt.set( "__index", lua.create_function( |lua, (t, k): (mlua::Table, mlua::Value)| -> mlua::Result<mlua::Value> {
            let env_str = lua.create_string(ENV)?;
            match k == mlua::Value::String(env_str) {
                true => t.raw_get( k ),
                false => {
                    let e: mlua::Table = t.raw_get(ENV)?;
                    e.get(k)
                },
            }
        } )?)?;
        wrapped_mt.set( "__newindex",lua.create_function( |_, (t, k, v): (mlua::Table, mlua::Value, mlua::Value)| -> mlua::Result<()> {
            let e: mlua::Table = t.raw_get(ENV)?;
            e.set(k, v)
        } )?)?;
        wrapped.set_metatable( Some(wrapped_mt) );

        let env_mt = lua.create_table()?;
        env_mt.set( "__index", globals.clone() )?;

        Ok(Lua{
            lua,
            globals,
            env_mt,
        })
    }
}

/// Just a simple wrapper for a Lua environment now
pub struct Env {
    table: mlua::Table,
}
impl Env {
    pub fn new( lua: &Lua ) -> mlua::Result<Self> {
        let table = lua.lua.create_table()?;
        table.set_metatable( Some(lua.env_mt.clone()) );
        Ok(Env{table})
    }

    pub fn set( &self, lua: &Lua ) -> mlua::Result<()> {
        lua.lua.globals().raw_set( ENV, self.table.clone() )
    }
}

/// Run tests with two environments
fn main() -> mlua::Result<()> {
    let lua = Lua::new()?;

    let env1 = Env::new( &lua )?;
    let env2 = Env::new( &lua )?;

    env1.set( &lua )?;
    lua.lua.load("
    assert( cat == nil )
    cat = 5
    assert( cat == 5 )
    cat = 7
    assert( cat == 7 )
    print('TEST1 OK')
    ").exec()?;

    env2.set( &lua )?;
    lua.lua.load("
    assert( cat == nil )
    cat = 9
    assert( cat == 9 )
    print('TEST2 OK')
    ").exec()?;

    env1.set( &lua )?;
    lua.lua.load("
    assert( cat == 7 )
    print('TEST3 OK')
    ").exec()?;

    lua.globals.raw_set( "notouchie", 1 )?;
    lua.lua.load("
    assert( notouchie == 1 )
    notouchie = 2
    assert( notouchie == 2 )
    notouchie = nil
    assert( notouchie == 1 )
    print('TEST4 OK')
    ").exec()?;

    let globals = lua.lua.globals();
    let cat: mlua::Integer = globals.get("cat")?;
    assert!( cat==7 );
    env2.set( &lua )?;
    let cat: mlua::Integer = globals.get("cat")?;
    assert!( cat==9 );
    println!("TEST5 OK");

    Ok(())
}
