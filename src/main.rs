use std::fmt::Debug;
use std::mem::{size_of, self};
use std::{ffi::*, slice};
use std::str;

use ash::vk::{DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT, DebugUtilsMessengerCallbackDataEXT};
use ash::{Entry, Instance};

use windows::Win32::Foundation::ERROR_VOLMGR_DISK_NOT_ENOUGH_SPACE;
use windows::{
    Win32::{UI::WindowsAndMessaging::*,
    System::LibraryLoader::{GetModuleHandleA},
    Foundation::{HWND, WPARAM, LPARAM, LRESULT}},
    core::*};

static mut EXIT : bool = false;
static mut MAIN_WINDOW_HANDLE : Option<HWND> = None;

static mut VK_INSTANCE : Option<Instance> = None;

static mut VALIDATION_LAYERS_ENABLED : bool = true;

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

        if main_window_handle.0 == 0 {
            panic!("Failed to create main window");
        }

        MAIN_WINDOW_HANDLE = Some(main_window_handle);

        ShowWindow(MAIN_WINDOW_HANDLE.unwrap(), SW_SHOWNORMAL);

        println!("*** Initializing Vulkan ***");

        // Load the Vulkan Library dynamically
        let vk_entry = Entry::load().unwrap();

        // Check support for required layers
        let required_layers = vec!(
            "VK_LAYER_KHRONOS_validation",
        );

        if !check_layer_support(&vk_entry, &required_layers) {
            panic!("Failed to find support for all specified layers!");
        }

        // First thing to do when initializing Vulkan is to create an "instance".
        // The instance is the connection between your application and the Vulkan library.
        
        // To create an instance, we have to fill out a struct with information about our application.
        // The data is optional, but it may provide some useful information to the driver in order to optimize
        // our application.
        let application_name = CString::new("Fantasy World").unwrap();
        let engine_name = CString::new("No Engine").unwrap();
        let application_info = ash::vk::ApplicationInfo::builder()
            .application_name(application_name.as_c_str())
            .application_version(ash::vk::make_api_version(1, 0, 0, 0))
            .engine_name(engine_name.as_c_str())
            .engine_version(ash::vk::make_api_version(1, 0, 0, 0))
            .api_version(ash::vk::API_VERSION_1_0)
            .build();

        // We also have to define an InstanceCreateInfo struct, which is required.
        // This struct tells the Vulkan driver which global extensions and validation layers
        // we want to use.
        let required_extensions : Vec<CString> = vec![
            CString::new("VK_KHR_surface").unwrap(),
            CString::new("VK_KHR_win32_surface").unwrap(),
            CString::new("VK_EXT_debug_utils").unwrap(),
        ];

        let required_extensions_api_call : Vec<*const i8> =
            required_extensions
            .iter()
            .map(|extension| extension.as_c_str().as_ptr())
            .collect();

        // TODO: Consider providing DebugUtilsMessengerCreateInfo to the next pointer, in order to enable debug messaging during instance creation
        let mut create_info = ash::vk::InstanceCreateInfo::default();
        create_info.p_application_info = &application_info;
        create_info.pp_enabled_extension_names = required_extensions_api_call.as_ptr();
        create_info.enabled_extension_count = required_extensions.len() as u32;

        // Right now the best way I've found to pass a reference to an array of C-style strings (*const i8 *const i8) is to take the array of Rust strings
        // and convert them to an Array of CStrings, because CString has a method to convert a String to a c-style string with nul termination.
        // Then I take that array of CString and get the pointer to null-terminated bytes of it.
        // The reason why I have to make a &str -> CString -> pointer to CString content is that I need the CString to stay alive to point to its content.
        // If I try to make a transform statement from &str -> pointer of nul-terminated bytes through a CString defined in the Map closure, it obviously fails because the CStrings
        // go out of scope at the end of their map.
        // I could obviously straight up create a CStr array from string literals that are nul-terminated, but one of my goals is to hide as many C-specific types and concerns
        // from the most abstracted layers of my code, going forward.
        let validation_layers_c_string : Vec<CString> = required_layers
            .iter()
            .map(|validation_layer| {
                CString::new(validation_layer.to_string()).unwrap()
            })
            .collect();

        let validation_layers_api_string : Vec<*const i8> = validation_layers_c_string.iter().map(|x| {
            x.as_bytes_with_nul().as_ptr() as *const i8
        })
        .collect();

        if VALIDATION_LAYERS_ENABLED {
            create_info.pp_enabled_layer_names = validation_layers_api_string.as_ptr();
            create_info.enabled_layer_count = validation_layers_api_string.len() as u32;
        }

        // Create the instance!
        VK_INSTANCE = Some(vk_entry.create_instance(&create_info, None).unwrap());
        let vk_instance_local = VK_INSTANCE.as_ref().unwrap();

        // Set up Vulkan Debug Messenger
        let vk_debug_messenger = ash::vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(ash::vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE | ash::vk::DebugUtilsMessageSeverityFlagsEXT::WARNING | ash::vk::DebugUtilsMessageSeverityFlagsEXT::ERROR)
            .message_type(ash::vk::DebugUtilsMessageTypeFlagsEXT::GENERAL | ash::vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION | ash::vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE)
            .pfn_user_callback(Some(debug_messenger_callback))
            .build();

        let debug_utils_extension = ash::extensions::ext::DebugUtils::new(&vk_entry, &vk_instance_local);

        let vk_debug_messenger = debug_utils_extension.create_debug_utils_messenger(&vk_debug_messenger, None).unwrap();

        let mut msg = MSG::default();
        while !EXIT {
            // PeekMessage retrieves messages associated with the window identified by the handle.
            // It returns a nonzero value is a message is available.
            // PM_REMOVE option means that messages are removed from the queue after processing by PeekMessage.
            if PeekMessageW(&mut msg, MAIN_WINDOW_HANDLE.unwrap(), 0, 0, PM_REMOVE) == windows::Win32::Foundation::TRUE {
                if msg.message == WM_QUIT {
                    EXIT = true;
                }

                // TranslateMessage is related to keyboard input.
                // It translates virtual key messages into character messages. These are then posted to the calling thread's message queue, to be read by the next PeekMessage function.
                // More precisely, the function produces WM_CHAR messages only for keys that are mapped to ASCII characters by the keyboard driver.
                // It always has to be called before DispatchMessage.
                TranslateMessage(&msg);

                // DispatchMessage dispatches a message to the window procedure.
                // That is, DispatchMessage calls the window procedure you provided in the window class.
                DispatchMessageW(&msg);
            } else {
                // RENDER HERE
            }
        }

        // Cleanup debug messenger
        debug_utils_extension.destroy_debug_utils_messenger(vk_debug_messenger, None);

        // Application cleanup
        vk_instance_local.destroy_instance(None);
    }
}

/*
 The Vulkan debug messenger callback is called 
*/
unsafe extern "system" fn debug_messenger_callback(
    message_severity : DebugUtilsMessageSeverityFlagsEXT,
    message_type : DebugUtilsMessageTypeFlagsEXT,
    p_callback_data : *const DebugUtilsMessengerCallbackDataEXT,
    p_user_data : *mut c_void) -> ash::vk::Bool32 {

    let raw_message = (*p_callback_data).p_message;
    let message_as_c_str = CStr::from_ptr(raw_message);

    let message_severity = match message_severity {
        DebugUtilsMessageSeverityFlagsEXT::ERROR => "Error",
        DebugUtilsMessageSeverityFlagsEXT::INFO => "Info",
        DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "Verbose",
        DebugUtilsMessageSeverityFlagsEXT::WARNING => "Warning",
        _ => "Unknown Severity",
    };

    println!("{}: {}", message_severity, message_as_c_str.to_str().unwrap());

    // The debug messenger callback should always return false.
    // True is reserved for layer development only.
    ash::vk::FALSE
}

fn check_layer_support(vk_entry: &Entry, layers_to_check : &Vec<&str>) -> bool {
    unsafe {
        let instance_layer_properties = vk_entry.enumerate_instance_layer_properties().unwrap();

        for layer_name in layers_to_check {
            let mut layer_found = false;

            for available_layer in &instance_layer_properties {
                let available_layer_name_slice = std::slice::from_raw_parts::<u8>(available_layer.layer_name.as_ptr() as *const u8, available_layer.layer_name.len());
                let available_layer_name = CStr::from_bytes_until_nul(available_layer_name_slice).unwrap()
                    .to_str().unwrap();

                let pislort = *layer_name;

                if available_layer_name == pislort {
                    layer_found = true;
                    break;
                }
            }

            if !layer_found {
                return false;
            }
        }

        true
    }
}

// window = handle to the window.
// message = the message code, for example WM_SIZE
// wparam + lparam = additional data pertaining to the message. The data depends on the message code.
extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            // WM_CLOSE is received if the user clicks the Close button in the window corner.
            // It gives you an opportunity to react before actually terminating your application.
            WM_CLOSE => {
                EXIT = true;
                DestroyWindow(MAIN_WINDOW_HANDLE.unwrap());
                LRESULT(0)
            },
            // If you don't handle a particular message in your procedure, you can pass the message to DefWindowProc.
            // This function performs default actions for messages.
            _ => DefWindowProcW(window, message, wparam, lparam),
        }
    }
}