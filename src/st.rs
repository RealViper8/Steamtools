
#[repr(C)]
pub struct LuaState {
    _private: [u8; 0]
}

#[repr(C)]
#[derive(Debug)]
pub struct Lua {
    pub state: *mut LuaState
}

unsafe extern "C" {
    pub fn init_lua() -> Lua;
}