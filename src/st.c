#define _USE_MATH_DEFINES
#include <lua.h>
#include <lualib.h>
#include <lauxlib.h>

#include <stdio.h>
#include <math.h>
#include <stdlib.h>

/// @brief Lua state
typedef struct {
    lua_State* state;
} Lua;

/// @brief Download function for lua plugin (FFI Rust)
/// @param L Lua State passed to rust
/// @return 
extern int download(lua_State* L);

void run_lua_file(char* filename) {
    lua_State* L = luaL_newstate();
    luaL_openlibs(L);
    lua_pushcfunction(L, download);
    lua_setglobal(L, "download");

    if (luaL_dofile(L, filename)) {
        fprintf(stderr, "Couldn't run file: %s\n", lua_tostring(L, -1));
        exit(1);
    }
    
    lua_close(L);
}

void load_lua_file(char* filename) {
    int status, result, i;
    lua_State * L = luaL_newstate();
    luaL_openlibs(L);

    status = luaL_loadfile(L, filename);
    if (status) {
        fprintf(stderr, "Couldn't load file: %s\n", lua_tostring(L, -1));
        exit(1);
    }
    
    lua_close(L);
}