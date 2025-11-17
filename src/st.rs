#![allow(unused)]

use std::ffi::{CString, c_int};
mod ffi {
    use core::ffi::c_char;
    use std::ffi::c_int;

    #[repr(C)]
    pub struct LuaState {
        _private: [u8; 0]
    }

    #[link(name="lua", kind="static")]
    unsafe extern "C" {
        pub fn luaL_newstate() -> *mut LuaState;
        pub fn luaL_openlibs(L: *mut LuaState);
        pub fn luaL_dostring(L: *mut LuaState, s: *const c_char) -> c_int;
        pub fn lua_close(L: *mut LuaState);

        pub fn lua_tolstring(L: *mut LuaState, index: c_int, len: *const usize) -> *const c_char;
        pub fn lua_pushboolean(L: *mut LuaState, b: c_int);
        pub fn lua_getglobal(L: *mut LuaState, name: *const c_char) -> c_int;
        pub fn lua_pcallk(L: *mut LuaState, args: c_int, nresults: c_int, errfunc: c_int, ctx: c_int, k: *const c_int) -> c_int;
        pub fn luaL_loadfilex(L: *mut LuaState, filename: *const c_char, mode: *const c_char) -> c_int;
    }

    unsafe extern "C" {
        pub(crate) fn run_lua_file(filename: *const c_char) -> c_int;
        pub(crate) fn set_flag(val: c_int);
        pub(crate) fn get_flag() -> c_int;
    }
}

pub fn stop_file() {
    unsafe { ffi::set_flag(1); };
}


pub fn start_file() {
    unsafe { ffi::set_flag(0); };
}

pub fn run_lua_file<T: Into<Vec<u8>>>(filename: T ) -> Option<()> {
    let s= CString::new(filename).unwrap();
    if unsafe {ffi::run_lua_file(s.as_ptr()) } != 0 {
        None
    } else {
        Some(())
    }
}
mod macros {
    use std::{ffi::{c_char, c_int}, ptr::null};

    use crate::st::ffi::{LuaState, lua_pcallk, luaL_loadfilex};

    pub fn lua_tostring(l: *mut LuaState, index: c_int) -> *const c_char {
        unsafe { super::ffi::lua_tolstring(l, index, null()) }
    }

    pub fn lua_pcall(l: *mut LuaState, args: c_int, nresults: c_int, errfunc: c_int) -> c_int {
        unsafe { lua_pcallk(l, args, nresults, errfunc, 0, null()) }
    }


    pub fn lua_loadfile(l: *mut LuaState, filename: *const c_char) -> c_int {
        unsafe { luaL_loadfilex(l, filename, null()) }
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn download(l: *mut ffi::LuaState) -> c_int {
    use std::{ffi::CStr, fs::File, io::Write};
    use macros::lua_tostring;
    use ffi::{lua_pushboolean};

    let url = unsafe { CStr::from_ptr(lua_tostring(l, 1)).to_str().unwrap() };
    let out_path = unsafe { CStr::from_ptr(lua_tostring(l, 2)).to_str().unwrap() };

    match reqwest::blocking::get(url) {
        Ok(resp) => {
            if let Ok(bytes) = resp.bytes() {
                let mut file = File::create(out_path).unwrap();
                file.write_all(&bytes).unwrap();
                unsafe { lua_pushboolean(l, 1) };
                return 1;
            }
        }
        Err(_) => {}
    }

    unsafe { lua_pushboolean(l, 0) };
    1
}

macro_rules! cstr {
    ($s:literal) => {
        std::ffi::CString::new($s).unwrap().as_ptr()
    };
}

#[cfg(test)]
mod tests {
    use std::{ffi::{CStr, CString}, fs, io::Write};

    use crate::st::{ffi::{lua_close, lua_getglobal, luaL_newstate, luaL_openlibs, run_lua_file}, macros::{lua_loadfile, lua_pcall, lua_tostring}};

    #[test]
    fn run() {
        let mut f = fs::File::create("test.lua").unwrap();
        f.write_all(b"print(\"test\")");
        f.flush();

        let s = CString::new("test.lua").unwrap();
        unsafe { run_lua_file(s.as_ptr()) };
        // let l = unsafe { luaL_newstate() };
        // let t = CString::new("Ui").unwrap();
        // unsafe { luaL_openlibs(l) };
        
        // if unsafe { lua_loadfile(l, s.as_ptr()) } != 0 {
        //     let err = unsafe { CStr::from_ptr(lua_tostring(l, -1)) };
        //     eprintln!("Error loading Lua file: {:?}", err);
        //     return;
        // }

        // lua_pcall(l, 0, 0, 0);
        // unsafe { lua_getglobal(l, t.as_ptr()) };
        // lua_pcall(l, 0, 0, 0);

        // unsafe { lua_close(l) };
    }
}