# Overview

Vulkan is a graphics API made by the Kronos Group.

The idea behind it is the same as behind Direct3D 12 and Metal, in that it allows you to better describe exactly what your application intends to do, which can lead to better performance and less surprising driver behaviour.

However the price you pay is a more verbose API, because every detail related to the graphics API needs to be set up from scratch by your application.

So, the payoff of less implicit behaviour is more explicit work by you. This is usually in contrast to APIs such as OpenGL, where a lot of initial decisions is taken behind the scenes by the implementation.

## Patterns

### Struct parameter passing

A lot of information in Vulkan is passed to functions in structs that you define before calling the function.

### Minimal Overhead

Vulkan operates on a design goal of minimal driver overhead.

Example: there is actually very limited error checking in the API by default. Mistakes such as setting enumerations to incorrect values or passing null pointers to required parameters is not explicitly handled and will simply result in a crash, or undefined behaviour.

In order to add things such as debug support, or error checking, you use a type of behaviour extension known as **layers**, which is code that is inserted into the call chain of a function which the layer is interested in.
