use std::mem::{size_of, self};

use windows::{
    Win32::{UI::WindowsAndMessaging::*,
    System::LibraryLoader::{GetModuleHandleA},
    Foundation::{HWND, WPARAM, LPARAM, LRESULT}},
    core::*};

fn main() {
    unsafe {
        // Retrieve module to the executable for the current process.
        // This HWND is used for many subsequent Win32 calls having to do with the current process.
        let handle = GetModuleHandleA(None)
            .expect("Failed to retrieve process handle.");

        // Every window in win32 is associated with a "window class".
        // This class has to be created and registered before we can use it when creating a window.
        // This class defines a set of behaviours several windows might have in common.
        // I suppose the equivalent would be the OOP term of a class, in that it defines a blueprint several instances might have in common.

        // Class name
        let window_class_name = w!("main_window");
    
        let mut main_window_class: WNDCLASSEXW = WNDCLASSEXW::default();
        main_window_class.cbSize = mem::size_of::<WNDCLASSEXW>() as u32; // The size of the struct. This member is required in WNDCLASSEXW.
        main_window_class.lpfnWndProc = Some(wndproc); // A function pointer to the window procedure function
        main_window_class.hInstance = handle.into(); // A handle to the executable
        main_window_class.lpszClassName = window_class_name; // name of the window class

        // Register the class
        // If the function fails, the return value will be zero.
        let register_class_status = RegisterClassExW(&main_window_class);
        if register_class_status == 0 {
            panic!("Failed to register window class");
        }

        // Create the main window
        let window_title = w!("Fantasy World");

        let main_window_handle = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            window_class_name,
            window_title,
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT, CW_USEDEFAULT, 800, 600,
            None,
            None,
            handle,
            None);

        ShowWindow(main_window_handle, SW_SHOWNORMAL);

        let mut msg = MSG::default();

        let mut exit = false;
        while !exit {

            // PeekMessage retrieves messages associated with the window identified by the handle.
            // It returns a nonzero value is a message is available.
            // PM_REMOVE option means that messages are removed from the queue after processing by PeekMessage.
            if PeekMessageW(&mut msg, main_window_handle, 0, 0, PM_REMOVE) == windows::Win32::Foundation::TRUE {
                // TranslateMessage is related to keyboard input.
                // It translates virtual key messages into character messages. These are then posted to the calling thread's message queue, to be read by the next PeekMessage function.
                // More precisely, the function produces WM_CHAR messages only for keys that are mapped to ASCII characters by the keyboard driver.
                // It always has to be called before DispatchMessage.
                TranslateMessage(&msg);

                // DispatchMessage dispatches a message to the window procedure.
                // That is, DispatchMessage calls the window procedure you provided in the window class.
                DispatchMessageW(&msg);

                if msg.message == WM_QUIT {
                    exit = true;
                }
            } else {
                // RENDER HERE
            }
        }
    }
}

// window = handle to the window.
// message = the message code, for example WM_SIZE
// wparam + lparam = additional data pertaining to the message. The data depends on the message code.
extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            // If you don't handle a particular message in your procedure, you can pass the message to DefWindowProc.
            // This function performs default actions for messages.
            _ => DefWindowProcW(window, message, wparam, lparam),
        }
    }
}