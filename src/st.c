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

static int stop_flag = 0;

void set_flag(int val) {
    stop_flag = val;
}

int get_flag() {
    return stop_flag;
}

void hook(lua_State *L, lua_Debug *ar) {
    if (stop_flag == 1) {
        luaL_error(L, "Execution stopped by user");
    }
}

int run_lua_file(char* filename) {
    lua_State* L = luaL_newstate();
    luaL_openlibs(L);
    lua_pushcfunction(L, download);
    lua_setglobal(L, "download");
    lua_sethook(L, hook, LUA_MASKCOUNT, 1000);

    if (luaL_dofile(L, filename)) {
        printf("Couldn't run file: %s\n", lua_tostring(L, -1));
        fflush(stdout);
        return 1;
    }
    
    lua_close(L);
    return 0;
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