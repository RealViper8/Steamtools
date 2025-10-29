#include <lua.h>
#include <lualib.h>
#include <lauxlib.h>

#include <stdio.h>
#include <stdlib.h>

/// @brief Lua state
typedef struct {
    lua_State* state;
} Lua;

Lua init_lua() {
    Lua lua;
    lua_State * L = luaL_newstate();
    lua.state = L;

    return lua;
}