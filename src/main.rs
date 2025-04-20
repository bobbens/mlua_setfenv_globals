fn main() -> mlua::Result<()> {
   #[allow(non_snake_case)]
   let L = unsafe{ mlua::ffi::luaL_newstate() };
   let lua = unsafe { mlua::Lua::init_from_ptr( L ) };

   unsafe {
      mlua::ffi::luaopen_base( L );
   }

   let t = lua.create_table()?;

   let m = lua.create_table()?;
   m.set("__index", lua.globals())?;
   t.set_metatable(Some(m));

   t.set( "testfunc", lua.create_function( |lua, ()| {
       let name: mlua::Value = lua.globals().get("__name")?;
       dbg!(name);
       Ok(())
    })?)?;

   t.set("__name", "foo" )?;
   let rk = lua.create_registry_value( t )?;

   lua.load("
function main ( str )
   print( __name, str )
   testfunc()
end
   ").exec()?;

   unsafe {
      mlua::ffi::lua_getglobal( L, c"main".as_ptr() );
      mlua::ffi::lua_pushstring( L, c"hello world".as_ptr() );
      mlua::ffi::lua_rawgeti( L, mlua::ffi::LUA_REGISTRYINDEX, rk.id().into() );
      mlua::ffi::lua_setfenv( L, -3 );
      if mlua::ffi::lua_pcall( L, 1, 0, 0 ) != 0{
          dbg!( std::ffi::CStr::from_ptr( mlua::ffi::luaL_tolstring( L, -1, std::ptr::null_mut() ) ) );
      }
   }

   Ok(())
}
