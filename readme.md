# Peer Practice

This project is a Rust workspace. The main application you’ll be building and running is the `peer_practice` crate.

---

## 1. Prerequisites

- **Rust toolchain** (via [rustup](https://rustup.rs/))
    - Recommended: latest stable.
- **Cargo** (installed with rustup)
- **Git** (to clone the repository)
- **direnv** (optional, but supported)
- **Nix** (optional, if you want to use the provided `flake.nix`)

---

## 2. Repository Layout (high level)

- `peer_practice/` – main application crate (binary).
- `peer_practice_shared/`, `peer_practice_messages/`, `peer_practice_server_services/` – workspace libraries/services.
- `web-leptos/` – web-facing crate (Leptos-based).
- `Cargo.toml` – workspace manifest (root).

You generally run commands from the **repository root** (the directory containing the *top-level* `Cargo.toml`).

---

## 3. Initial Setup

1. **Clone the repository**

   ```bash
   git clone <repo-url> peer_practice
   cd peer_practice
   ```

2. **(Optional) Enable direnv**

   If you use `direnv` and want to load environment from `.envrc`:

   ```bash
   direnv allow
   ```

3. **Configuration file**

   If there is no `config.toml` yet, create one from the provided backup template:

   ```bash
   cp config.bak.toml config.toml
   ```

   Then open `config.toml` and adjust any settings (ports, database URLs, API keys, email credentials, etc.) as needed.

---

## 4. Building

From the repository root:

- **Debug build (fast, for development)**

  ```bash
  cargo build
  ```

- **Release build (optimized, for deployment)**

  ```bash
  cargo build --release
  ```

Because the workspace’s `default-members` includes `peer_practice`, running `cargo build` at the root will build the
main service along with its dependencies.

If you want to build everything in the workspace:

```

bash cargo build --workspace``` 

---

## 5. Running the Main Service

From the repository root:

- **Run in debug mode**

  ```bash
  cargo run
  ```

This uses the workspace default member, which is the `peer_practice` binary.

- **Explicitly run the `peer_practice` crate** (equivalent, but more explicit):

  ```bash
  cargo run -p peer_practice
  ```

- **Run in release mode**

  ```bash
  cargo run --release -p peer_practice
  ```

When the service starts, it will read configuration from `config.toml` (and any relevant environment variables, if the
code is written to support that) and start listening on the configured port(s).

---

## 6. Running the Web / Leptos Component (if used)

If your setup uses the `web-leptos` crate as a separate service, you can build or run it explicitly:

- **Build only the web crate**

  ```bash
  cargo build -p web-leptos
  ```

- **Run the web crate**

  ```bash
  cargo run -p web-leptos
  ```

Depending on how routing is configured, the web service may talk to the `peer_practice` backend over HTTP/WebSockets.
Ensure the backend (`peer_practice`) is running first if the web front-end depends on it.

---

## 7. Using Nix (Optional)

If you prefer to use Nix for reproducible builds and dev environments, there is a `flake.nix`:

```

bash
Enter the Nix development shell (if using flakes)
nix develop
Inside the shell, build and run as usual
cargo build cargo run``` 

---

## 8. Running Tests

You can run tests across the entire workspace:
```

bash cargo test```

Or limit to a specific crate:

```

bash cargo test -p peer_practice cargo test -p peer_practice_shared
etc.``` 

---

## 9. Common Development Commands

- Format code:

  ```bash
  cargo fmt
  ```

- Lint (if `clippy` is installed):

  ```bash
  cargo clippy --workspace --all-targets
  ```

- Run only the main service (default):

  ```bash
  cargo run
  ```

---

## 10. Troubleshooting

- **Missing toolchain / Rust too old**  
  Install or update via:

  ```bash
  rustup update
  ```

- **Dependency download failures**  
  Try:

  ```bash
  cargo clean
  cargo build
  ```

- **Configuration errors**  
  Double-check `config.toml` and any required environment variables. If the server fails to start, read the error
  message in the logs; typically it will indicate which setting is invalid or missing.

---


If you share your actual `config.toml` schema or any error messages you run into,
I can add a short “Configuration” or “FAQ” section tailored to your deployment.
