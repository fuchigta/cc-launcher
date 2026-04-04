//! Windows Explorer context menu COM shell extension.
//!
//! Implements `IExplorerCommand` so that "cc-launcherで開く" appears in the
//! Windows 11 modern context menu (not only in "Show more options").
//!
//! CLSID: {4CC3A7F2-1B5E-4D9A-8F6C-3E2D1A4B5C7E}

#![allow(non_snake_case)]

use std::ffi::c_void;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::sync::OnceLock;

use windows::core::{
    implement, Error, IUnknown, Interface, Ref, Result, BOOL, GUID, HRESULT, PWSTR,
};
use windows::Win32::Foundation::{CLASS_E_NOAGGREGATION, E_FAIL, E_NOTIMPL, HMODULE, S_FALSE};
use windows::Win32::System::Com::{
    CoTaskMemAlloc, CoTaskMemFree, IBindCtx, IClassFactory, IClassFactory_Impl,
};
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows::Win32::UI::Shell::{
    IEnumExplorerCommand, IExplorerCommand, IExplorerCommand_Impl, IShellItemArray,
    SIGDN_FILESYSPATH,
};

/// CLSID for this COM class — must match registry and sync_context_menu_registry.
pub const CLSID_CC_LAUNCHER_MENU: GUID = GUID {
    data1: 0x4CC3A7F2,
    data2: 0x1B5E,
    data3: 0x4D9A,
    data4: [0x8F, 0x6C, 0x3E, 0x2D, 0x1A, 0x4B, 0x5C, 0x7E],
};

static DLL_MODULE: OnceLock<usize> = OnceLock::new();

// ---------------------------------------------------------------------------
// DLL entry points
// ---------------------------------------------------------------------------

/// Capture our HMODULE so we can resolve sibling paths later.
#[no_mangle]
unsafe extern "system" fn DllMain(
    h_instance: HMODULE,
    dw_reason: u32,
    _lp_reserved: *const c_void,
) -> BOOL {
    if dw_reason == 1 {
        // DLL_PROCESS_ATTACH
        let _ = DLL_MODULE.set(h_instance.0 as usize);
    }
    BOOL(1)
}

/// Standard COM class-object factory entry point.
#[no_mangle]
unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    *ppv = std::ptr::null_mut();
    if *rclsid != CLSID_CC_LAUNCHER_MENU {
        return HRESULT(0x80040111u32 as i32); // CLASS_E_CLASSNOTAVAILABLE
    }
    let factory: IClassFactory = CcLauncherCommandFactory {}.into();
    Interface::query(&factory, riid, ppv)
}

/// Allow Explorer to unload; S_FALSE = "keep loaded" (simplest approach).
#[no_mangle]
extern "system" fn DllCanUnloadNow() -> HRESULT {
    S_FALSE
}

// ---------------------------------------------------------------------------
// IClassFactory
// ---------------------------------------------------------------------------

#[implement(IClassFactory)]
struct CcLauncherCommandFactory;

impl IClassFactory_Impl for CcLauncherCommandFactory_Impl {
    fn CreateInstance(
        &self,
        punkouter: Ref<'_, IUnknown>,
        riid: *const GUID,
        ppv: *mut *mut c_void,
    ) -> Result<()> {
        unsafe {
            *ppv = std::ptr::null_mut();
        }
        if !punkouter.is_null() {
            return Err(Error::from(CLASS_E_NOAGGREGATION));
        }
        let command: IExplorerCommand = CcLauncherCommand {}.into();
        unsafe { Interface::query(&command, riid, ppv).ok() }
    }

    fn LockServer(&self, _flock: BOOL) -> Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// IExplorerCommand
// ---------------------------------------------------------------------------

#[implement(IExplorerCommand)]
struct CcLauncherCommand;

impl IExplorerCommand_Impl for CcLauncherCommand_Impl {
    fn GetTitle(&self, _psiitemarray: Ref<'_, IShellItemArray>) -> Result<PWSTR> {
        alloc_pwstr("cc-launcherで開く")
    }

    fn GetIcon(&self, _psiitemarray: Ref<'_, IShellItemArray>) -> Result<PWSTR> {
        match get_exe_path() {
            Some(exe) => alloc_pwstr(&format!("{},0", exe.display())),
            None => Err(Error::from(E_FAIL)),
        }
    }

    fn GetToolTip(&self, _psiitemarray: Ref<'_, IShellItemArray>) -> Result<PWSTR> {
        Err(Error::from(E_NOTIMPL))
    }

    fn GetCanonicalName(&self) -> Result<GUID> {
        Ok(CLSID_CC_LAUNCHER_MENU)
    }

    fn GetState(&self, _psiitemarray: Ref<'_, IShellItemArray>, _foktobeslow: BOOL) -> Result<u32> {
        Ok(0) // ECS_ENABLED
    }

    fn GetFlags(&self) -> Result<u32> {
        Ok(0) // ECF_DEFAULT
    }

    fn EnumSubCommands(&self) -> Result<IEnumExplorerCommand> {
        Err(Error::from(E_NOTIMPL))
    }

    fn Invoke(
        &self,
        psiitemarray: Ref<'_, IShellItemArray>,
        _pbc: Ref<'_, IBindCtx>,
    ) -> Result<()> {
        // Catch any panics to avoid crashing Explorer.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = launch_cc_launcher(&psiitemarray);
        }));
        Ok(())
    }
}

/// Extract the folder path from the shell item array and launch cc-launcher.
fn launch_cc_launcher(psiitemarray: &Ref<'_, IShellItemArray>) -> Result<()> {
    let folder_path = unsafe {
        let items = match psiitemarray.as_ref() {
            Some(v) => v,
            None => return Err(Error::from(E_FAIL)),
        };
        let item = items.GetItemAt(0)?;
        let path_pwstr = item.GetDisplayName(SIGDN_FILESYSPATH)?;
        let path_str = pwstr_to_string(path_pwstr.0);
        CoTaskMemFree(Some(path_pwstr.0 as *const c_void));
        path_str
    };

    let exe_path = get_exe_path().ok_or(Error::from(E_FAIL))?;
    std::process::Command::new(&exe_path)
        .args(["--directory", &folder_path])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .spawn()
        .map_err(|_| Error::from(E_FAIL))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Allocate a CoTask-owned wide string. Explorer calls CoTaskMemFree on it.
fn alloc_pwstr(s: &str) -> Result<PWSTR> {
    let wide: Vec<u16> = s.encode_utf16().chain(Some(0u16)).collect();
    unsafe {
        let ptr = CoTaskMemAlloc(wide.len() * std::mem::size_of::<u16>()) as *mut u16;
        if ptr.is_null() {
            return Err(Error::from(windows::Win32::Foundation::E_OUTOFMEMORY));
        }
        std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
        Ok(PWSTR(ptr))
    }
}

/// Convert a raw UTF-16 pointer to a Rust String.
unsafe fn pwstr_to_string(ptr: *mut u16) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let mut len = 0usize;
    let mut p = ptr;
    while *p != 0 {
        p = p.add(1);
        len += 1;
    }
    String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len)).to_string()
}

/// Return the directory that contains this DLL.
fn get_dll_dir() -> Option<PathBuf> {
    let module_val = DLL_MODULE.get()?;
    let mut buf = [0u16; 260]; // MAX_PATH
    let module = HMODULE(*module_val as *mut std::ffi::c_void);
    let len = unsafe { GetModuleFileNameW(Some(module), &mut buf) };
    if len == 0 {
        return None;
    }
    let path = String::from_utf16_lossy(&buf[..len as usize]).to_string();
    PathBuf::from(path).parent().map(|p| p.to_path_buf())
}

/// Locate cc-launcher.exe in the same directory as this DLL.
fn get_exe_path() -> Option<PathBuf> {
    let dir = get_dll_dir()?;
    let exe = dir.join("cc-launcher.exe");
    if exe.exists() {
        Some(exe)
    } else {
        None
    }
}
