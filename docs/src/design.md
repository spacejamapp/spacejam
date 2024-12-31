# Design

SpaceJam is a modular and extensible system that adheres to the JAM specification while providing flexibility for custom implementations.

## Overview

The codebase is organized into two main components: the core components under `@core` and the SpaceJam architecture under `@spacejam`. This structure allows for a modular and extensible design that adheres to the JAM specification while providing flexibility for custom implementations.

### Core Components

- The `core` directory contains the fundamental components that implement the JAM specification. These components are essential for the functionality of the system and include various modules that define the core logic, data structures, and protocols.

### SpaceJam Architecture

- The `spacejam` directory introduces a flexible architecture that allows for various implementations of the `Validator` and `Storage` traits defined in the `core` components. This architecture is designed to facilitate the integration of different validator and storage mechanisms, enabling users to customize their implementations based on specific requirements.

### Custom Implementations

- Users can create their own implementations of the `Validator` and `Storage` traits. This flexibility allows for a wide range of use cases, from simple local validators to more complex distributed storage solutions. The `spacejam` runner serves as the entry point for executing these custom implementations, providing a cohesive environment for testing and deploying different configurations.

## Key Features

- **Modular Design**: The codebase is organized into modules, making it easy to navigate and understand the relationships between different components. Each module encapsulates specific functionality, promoting separation of concerns.

- **Extensibility**: The architecture is designed to be extensible, allowing developers to add new features or modify existing ones without disrupting the overall system. This is achieved through the use of traits and interfaces that define clear contracts for behavior.

- **Integration with External Libraries**: The codebase leverages external libraries (e.g., `serde`, `crypto`, `rocksdb`) to enhance functionality, such as serialization, cryptographic operations, and storage solutions. This integration allows for robust and efficient implementations.

## Conclusion

The design of the codebase under the `@crates` directory emphasizes modularity, extensibility, and adherence to the JAM specification. By providing a flexible architecture in the `@spacejam` directory, developers are empowered to create custom validator and storage implementations, fostering innovation and adaptability within the ecosystem.
