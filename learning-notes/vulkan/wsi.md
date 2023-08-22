# Window System Integration (WSI)

The Vulkan API itself is a platform-agnostic API.

Vulkan in itself can be used without displaying results visually to a user.

Because of that, displaying results visuall to a user through something like a window is given through optional Vulkan extensions.

There are platform-specific WSI extensions for native types (such as Windows, Mac OS, Android, etc...).

## WSI Surface

https://registry.khronos.org/vulkan/specs/1.3-extensions/html/chap34.html#_wsi_platform

The WSI surface object is an abstraction of native platform surface or window objects. These are represented by the **VkSurfaceKHR** handle.

The **VK_KHR_Surface** extension declares the **VkSurfaceKHR** object.

From the application's perspective, this is an opaque handle, meaning you don't know the underlying specific type or implementation.

For Windows, the platform-specific extension for the underlying surface implementation is **VK_KHR_win32_surface**.