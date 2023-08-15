use ash::vk::HINSTANCE;
use windows::{
    Win32::{UI::WindowsAndMessaging::*,
    System::LibraryLoader::{GetModuleHandleA}, Foundation::GetLastError},
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
        main_window_class.lpfnWndProc = None; // A function pointer to the window procedure function
        main_window_class.hInstance = handle.into(); // A handle to the executable
        main_window_class.lpszClassName = window_class_name; // name of the window class

        // Register the class
        // If the function fails, the return value will be zero.
        let register_class_status = RegisterClassExW(&main_window_class);
        if register_class_status == 0 {
            let error_code = GetLastError().unwrap_err();
            panic!("Failed to register window class.");
        }
    }
}
