# Overview

Vulkan is a graphics API made by the Kronos Group.

The idea behind it is the same as behind Direct3D 12 and Metal, in that it allows you to better describe exactly what your application intends to do, which can lead to better performance and less surprising driver behaviour.

However the price you pay is a more verbose API, because every detail related to the graphics API needs to be set up from scratch by your application.

So, the payoff of less implicit behaviour is more explicit work by you. This is usually in contrast to APIs such as OpenGL, where a lot of initial decisions is taken behind the scenes by the implementation.

## Patterns

### Struct parameter passing

A lot of information in Vulkan is passed to functions in structs that you define before calling the function.