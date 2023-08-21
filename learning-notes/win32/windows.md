# Windows

## Windows Messaging

Windows uses a message-passing model to UI applications. The operating system communicates with your application window by passing messages to it.
A message is a numeric code that designates a particular event. Additionally, some events can have data associated with it.

Messages can include events from the user (such as mouse clicks, key strokes, touch-screen gestures) or messages from the OS, such as new hardware being
plugged in, or Windows entering a lower-power state.

In order to pass a message, Windows will call the application's window procedure registered for that particular window. This is the wind proc function.

For each thread that creates a window, the OS creates a queue for window messages. This queue holds messages for all the windows that are created on that thread.
The queue itself cannot be manipulated by your program, but you can get messages from it through Win32 functions.

## Message Loop

An a