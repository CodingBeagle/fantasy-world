# Layers & Extensions

## Extensions

Extensions may define new Vulkan commands, structures, and enumerants.

There a different types of extensions.

- Instance Extensions
  - Adds new instance-level functionality to the API, outside of the core specification.
  - Commands that enumerate instance properties, or accept a VkInstance object as a parameter, are considered instance-level functionality.
- Device Extensions
  - Adds new device-level functionality to the API, outside of the core specification.
  - Commands that dispatch from a VkDevice object or a child object of a VkDevice, or take any of them as a parameter, are considered device-level functionality.

## Layers

Layers inserts themselves into the call chain for Vulkan commands for which the layer is interested in.

Layers can be used to extend the base behaviour of Vulkan beyond what is required by the specification - such as call logging, traving, validation, or providing additional extensions.

Example: An implementation is not expected to check that the value of enums used by the application fall within allowed ranges. Instead, a validation layer would do those checks and flag issues. This avoids a performance penanlty during production use of the application because those layers would not be enabled in production.