# Safe Server Bot

Safe Server is a Discord bot built with Rust and the Serenity framework designed to enhance server management and safety. It provides detailed reports on roles within the server, highlighting any dangerous permissions that could potentially disrupt the server's operation.

## Features

- **Full Role Report**: Generates a detailed report for a specified role, including the percentage of members with the role and a list of any dangerous permissions.
- **Info Summary**: Provides a summary of all roles in the server that exceed a specified threshold of dangerous permissions.

## Setup

To set up the bot, follow these steps:

1. **Clone the Repository**:
   ```sh
   git clone https://github.com/FloppaSupreme/safe_server.git
   cd safe-server
    ```
2. **Install Dependencies**:
    ```sh
    cargo build --release
    ```

3. **Create a Bot Account**

4. **Create a `.env` File**:
    ```sh
    touch .env
    ```

5. **Add bot token to `.env`
   ```
   DISCORD_TOKEN=TOKEN_HERE
   ```

6. **Run the Bot**
    ```sh
    RUST_LOG=safe_server=debug cargo run
    ```
    The `RUST_LOG` environment variable is optional, but it will enable logging for the bot