# Ultor

## Overview

This is a **Discord bot** used for managing **sponsors** on the Space Station 14 server **STALKER 14**.  
It is written in **pure [`serenity`](https://github.com/serenity-rs/serenity)** without the use of higher-level frameworks, meaning the internal structure is **low-level** and **manual** by design.  
As such, the codebase may be more challenging to understand or extend for developers unfamiliar with **Rust**.

## Architecture Overview

The bot’s structure is deliberately kept modular and clean:

- **`src/services/`** — All non-Discord external interactions (e.g., database, API calls) are encapsulated in services.
- **`src/bot/commands/`** — All Discord slash commands are implemented here.
- **`src/lib.rs`** — Central coordination:
  - Use `command_definitions()` to register commands
  - Use `initialize_services()` to initialize all external service instances

This strict separation ensures maintainability across components.

## Deployment

This repository is **not intended for direct production use without Docker**.
To ensure consistent deployment and environment isolation, a **fully configured Dockerfile** is provided.
In addition, **CI/CD pipelines** are configured to automatically build and deploy the latest version of the bot as needed.

## License

This project is distributed under the [MIT License](LICENSE.TXT).  
You are free to use, modify, and redistribute with proper attribution.


