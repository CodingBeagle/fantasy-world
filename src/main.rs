use std::collections::HashSet;
use std::fmt::Debug;
use std::mem::{size_of, self};
use std::{ffi::*, slice};
use std::str;

use ash::vk::{DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT, DebugUtilsMessengerCallbackDataEXT, QueueFamilyProperties, QueueFamilyQueryResultStatusPropertiesKHR, DeviceQueueCreateInfo};
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

static mut REQUIRED_VALIDATION_LAYERS : Option<Vec<&str>> = None;

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
        REQUIRED_VALIDATION_LAYERS = Some(vec!(
            "VK_LAYER_KHRONOS_validation",
        ));

        if !check_layer_support(&vk_entry, &REQUIRED_VALIDATION_LAYERS.as_ref().unwrap()) {
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
        let validation_layers_c_string : Vec<CString> = REQUIRED_VALIDATION_LAYERS
            .as_ref()
            .unwrap()
            .iter()
            .map(|validation_layer| {
                CString::new(validation_layer.to_string()).unwrap()
            })
            .collect();

        let validation_layers_api_string : Vec<*const i8> = create_array_of_string_pointers(&validation_layers_c_string);

        if VALIDATION_LAYERS_ENABLED {
            create_info.pp_enabled_layer_names = validation_layers_api_string.as_ptr();
            create_info.enabled_layer_count = validation_layers_api_string.len() as u32;
        }

        // Create the instance!
        VK_INSTANCE = Some(vk_entry.create_instance(&create_info, None).unwrap());
        let vk_instance_local = VK_INSTANCE.as_ref().unwrap();

        // Create window surface
        // The surface handle itself will have its platform-specifics hidden away from us.
        // However, surface creation is platform specific.
        // For the sake of windows, it needs the concepts of HWND and HMODULE, which is win32 API concepts.
        let mut win32_surface_creation_info = ash::vk::Win32SurfaceCreateInfoKHR::default();
        win32_surface_creation_info.hwnd = MAIN_WINDOW_HANDLE.as_ref().unwrap().0 as *const c_void;
        win32_surface_creation_info.hinstance = handle.0 as *const c_void;

        let win32_surface_extensions = ash::extensions::khr::Win32Surface::new(&vk_entry, &vk_instance_local);

        let surface = win32_surface_extensions.create_win32_surface(&win32_surface_creation_info, None).unwrap();
        let surface_fn = ash::extensions::khr::Surface::new(&vk_entry, vk_instance_local);

        // Set up Vulkan Debug Messenger
        let vk_debug_messenger = ash::vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(ash::vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE | ash::vk::DebugUtilsMessageSeverityFlagsEXT::WARNING | ash::vk::DebugUtilsMessageSeverityFlagsEXT::ERROR)
            .message_type(ash::vk::DebugUtilsMessageTypeFlagsEXT::GENERAL | ash::vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION | ash::vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE)
            .pfn_user_callback(Some(debug_messenger_callback))
            .build();

        let debug_utils_extension = ash::extensions::ext::DebugUtils::new(&vk_entry, &vk_instance_local);

        let vk_debug_messenger = debug_utils_extension.create_debug_utils_messenger(&vk_debug_messenger, None).unwrap();

        // Pick GPU
        let picked_gpu = pick_physical_device(&vk_instance_local, surface, &surface_fn);
        let queue_family_indices = find_queue_families(picked_gpu, vk_instance_local, surface, &surface_fn);

        // Create logical device
        let logical_device = create_logical_device(vk_instance_local, picked_gpu, surface, &surface_fn);

        // Retrieve queue handle for the graphics family queue
        let graphics_family_queue_handle = logical_device.get_device_queue(queue_family_indices.graphics_family.unwrap(), 0);

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

        // Cleanup logical device
        logical_device.destroy_device(None);

        // Cleanup win32 surface
        // It has to be destroyed before the instance.
        surface_fn.destroy_surface(surface, None);

        // Application cleanup
        vk_instance_local.destroy_instance(None);
    }
}

fn create_array_of_string_pointers(c_string_collection : &Vec<CString>) -> Vec<*const i8> {
    c_string_collection.iter().map(|x| x.as_bytes_with_nul().as_ptr() as *const i8).collect()
}

unsafe fn create_logical_device(
    vk_instance : &ash::Instance,
    physical_device : ash::vk::PhysicalDevice,
    surface : ash::vk::SurfaceKHR,
    surface_fn : &ash::extensions::khr::Surface) -> ash::Device {
    let queue_family_indices = find_queue_families(physical_device, vk_instance, surface, &surface_fn);

    // Vulkan lets you assign priorities to queues to influence the scheduling of command buffer execution using floating point numbers between 0.0 and 1.0.
    let queue_priorities : Vec<f32> = vec![1.0];
    let mut queue_create_infos : Vec<DeviceQueueCreateInfo> = vec!();
    for graphic_queue_index in queue_family_indices.get_unique_family_indices() {
        let mut queue_create_info = ash::vk::DeviceQueueCreateInfo::default();
        queue_create_info.queue_family_index = graphic_queue_index;
        queue_create_info.queue_count = 1;
        queue_create_info.p_queue_priorities = queue_priorities.as_ptr();
        queue_create_infos.push(queue_create_info);
    }

    // We also need to specify device features we'd like to use.
    let device_features = ash::vk::PhysicalDeviceFeatures::default();

    // Now we can create the logical device
    let mut logical_device_create_info = ash::vk::DeviceCreateInfo::default();
    logical_device_create_info.p_queue_create_infos = queue_create_infos.as_ptr();
    logical_device_create_info.queue_create_info_count = queue_create_infos.len() as u32;
    logical_device_create_info.p_enabled_features = &device_features;

    // Previous implementations of Vulkan made a distinction between instance and device specific validation layers.
    // This is no longer the case. In up-to-date implementations enabled_layer_count and p_enabled_layer_names will be ignored.
    // However it can be good to set for backwards compatability.
    if VALIDATION_LAYERS_ENABLED {
        logical_device_create_info.pp_enabled_layer_names = create_array_of_string_pointers(&vec![CString::new("VK_LAYER_KHRONOS_validation").unwrap()]).as_ptr();
        logical_device_create_info.enabled_layer_count = REQUIRED_VALIDATION_LAYERS.as_ref().unwrap().len() as u32;
    }

    vk_instance.create_device(physical_device, &logical_device_create_info, None).unwrap()
}

unsafe fn pick_physical_device(
    vk_instance : &ash::Instance,
    surface : ash::vk::SurfaceKHR,
    surface_fn : &ash::extensions::khr::Surface) -> ash::vk::PhysicalDevice {
    let physical_devices = vk_instance.enumerate_physical_devices().unwrap();

    if physical_devices.is_empty() {
        panic!("Failed to find any physical GPU!");
    }

    // Pick the first suitable device
    for physical_device in physical_devices {
        if is_device_suitable(physical_device, vk_instance, surface, &surface_fn) {
            return physical_device;
        }
    }

    panic!("Unable to find any suitable GPU!");
}

// For now, I'm not doing anything fancy in determining what exactly I need. As long as it's a GPU!
unsafe fn is_device_suitable(
    physical_device : ash::vk::PhysicalDevice,
    vk_instance : &ash::Instance,
    surface : ash::vk::SurfaceKHR,
    surface_fn : &ash::extensions::khr::Surface) -> bool {
    // Basic device properties
    let physical_device_properties = vk_instance.get_physical_device_properties(physical_device);
    println!("Checking device {}...", CStr::from_ptr(physical_device_properties.device_name.as_ptr()).to_str().unwrap());

    // Support for optional features
    let physical_device_features = vk_instance.get_physical_device_features(physical_device);

    find_queue_families(physical_device, vk_instance, surface, &surface_fn).graphics_family.is_some()
}

#[derive(Default)]
struct QueueFamilyIndices {
    graphics_family: Option<u32>,
    present_family: Option<u32>
}

impl QueueFamilyIndices {
    // TODO: I dunno... this is not pretty
    pub fn get_unique_family_indices(&self) -> Vec<u32> {
        let mut indices : HashSet<u32> = HashSet::new();
        
        if self.graphics_family.is_some() {
            indices.insert(self.graphics_family.unwrap());
        }

        if self.present_family.is_some() {
            indices.insert(self.present_family.unwrap());
        }

        indices.iter().map(|x| { *x }).collect()
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

unsafe fn find_queue_families(
    physical_device : ash::vk::PhysicalDevice,
    vk_instance : &ash::Instance,
    surface : ash::vk::SurfaceKHR,
    surface_fn : &ash::extensions::khr::Surface) -> QueueFamilyIndices {
    let mut queue_family_indices = QueueFamilyIndices::default();

    let queue_families = vk_instance.get_physical_device_queue_family_properties(physical_device);

    let mut index_counter = 0;
    for queue_family in queue_families {
        if queue_family.queue_flags.contains(ash::vk::QueueFlags::GRAPHICS) {
            queue_family_indices.graphics_family = Some(index_counter);
        }

        let does_support_surface_present = surface_fn.get_physical_device_surface_support(physical_device, index_counter, surface).unwrap();
        if does_support_surface_present {
            queue_family_indices.present_family = Some(index_counter);
        }

        if queue_family_indices.is_complete() {
            break;
        }

        index_counter += 1;
    }

    queue_family_indices
}

fn check_layer_support(vk_entry: &Entry, layers_to_check : &Vec<&str>) -> bool {
    unsafe {
        let instance_layer_properties = vk_entry.enumerate_instance_layer_properties().unwrap();

        for layer_name in layers_to_check {
            let mut layer_found = false;

            for available_layer in &instance_layer_properties {
                let available_layer_name_slice = std::slice::from_raw_parts::<u8>(available_layer.layer_name.as_ptr() as *const u8, available_layer.layer_name.len());
                let available_layer_name = CStr::from_bytes_until_nul(available_layer_name_slice).unwrap().to_str().unwrap();

                let layer_name_to_check = *layer_name;

                if available_layer_name == layer_name_to_check {
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