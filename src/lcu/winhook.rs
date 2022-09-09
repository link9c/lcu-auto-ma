use winapi;
use winapi::shared::windef::HHOOK;
use winapi::um::winuser;
use winapi::um::winuser::SetWindowsHookExA;
use winapi::um::winuser::UnhookWindowsHookEx;
use winapi::um::winuser::KBDLLHOOKSTRUCT;
use winapi::um::winuser::{HC_ACTION, WH_KEYBOARD_LL};

use super::api::ApiClient;

static mut HOOK_HANDLE: Option<HHOOK> = None;

pub fn loop_send_by_key() {
    // register winapi hook to listen keyboard input.
    let hook_id =
        unsafe { SetWindowsHookExA(WH_KEYBOARD_LL, Some(hook_callback), std::ptr::null_mut(), 0) };

    println!("hook_id: {:?}", hook_id);

    // If for some reason hook action is not a keyboard reson it will be checked in hook_callback
    unsafe {
        HOOK_HANDLE = Some(hook_id);
    }

    unsafe {
        let msg: winuser::LPMSG = std::ptr::null_mut();
        while winuser::GetMessageA(msg, std::ptr::null_mut(), 0, 0) > 0 {
            winuser::TranslateMessage(msg);
            winuser::DispatchMessageA(msg);
        }
    }

    unsafe {
        UnhookWindowsHookEx(hook_id);
    }
}

extern "system" fn hook_callback(code: i32, w_param: usize, l_param: isize) -> isize {
    if code < HC_ACTION {
        unsafe {
            // Send hook to next thread
            if let Some(hook_id) = HOOK_HANDLE {
                winuser::CallNextHookEx(hook_id, code, w_param, l_param);
            } else {
                0;
            }
        }
    }

    let keypress: KBDLLHOOKSTRUCT = unsafe { *(l_param as *mut KBDLLHOOKSTRUCT) };

    if code == HC_ACTION && w_param == 257 {
        println!(
            "hook_callback {}, {}, {}, {:?}",
            code, w_param, l_param, keypress.vkCode
        );
        let key = from_code_to_char(keypress.vkCode);
        println!("Was pressed {}", key);
        if keypress.vkCode == 112 {
            let mut api = ApiClient::default();
            api.init_client();

            let s = api.get_gameflow_phase_block();

            println!("{:?}", s);
        }
    }
    0
}

/// convert u32 to char
fn from_code_to_char(code: u32) -> String {
    if code >= 65 && code <= 90 {
        (code as u8 as char).to_string()
    } else if code >= 48 && code <= 57 {
        (code as u8 as char).to_string()
    } else {
        return format!("Unknow code: {}", code);
    }
}
