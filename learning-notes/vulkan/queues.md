# Queues

Vulkan is built up around the concept of **devices** exposing **queues**, which can be used to process work asynchronously.

Queues are categorized into **families**, with each family supporting one or more types of functionality.

Queues may support the following type of functionality:

- Graphics
- Compute
- Video decode
- Video encode
- Protected memory management
- Sparse memory management
- Transfer
