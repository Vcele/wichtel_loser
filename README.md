# 🎄 Wichtel Loser

A festive Secret Santa (Wichteln) randomizer web application built with Rust and Actix-web.

## Features

- **Create Events**: Easily create new Wichteln events with unique invite codes
- **Join via Link**: Participants can join using a simple invite link
- **Random Assignments**: Automatically assigns Secret Santa pairs ensuring no one gets themselves
- **Cookie-based Identity**: Remembers participants via browser cookies
- **Fuzzy Name Search**: If cookies are lost, participants can find themselves via fuzzy search
- **Beautiful UI**: Festive, responsive design with snowfall animations

## Using the Nix Flake

This project uses Nix flakes for reproducible builds and deployments.

### Prerequisites

- [Nix](https://nixos.org/download.html) with flakes enabled

To enable flakes, add this to your `~/.config/nix/nix.conf`:
```
experimental-features = nix-command flakes
```

### Quick Start

#### Run directly without installing

```bash
# Run the application directly from GitHub
nix run github:Vcele/wichtel_loser

# Or from a local clone
nix run .
```

The server will start at `http://127.0.0.1:8080` by default.

#### Enter development shell

```bash
# Enter a shell with all build dependencies
nix develop

# Then build and run with cargo
cargo run
```

#### Build the package

```bash
# Build the package
nix build

# The binary will be available at ./result/bin/wichtel_loser
./result/bin/wichtel_loser
```

### Configuration

Set the `BIND_ADDRESS` environment variable to change the server address:

```bash
BIND_ADDRESS="0.0.0.0:3000" nix run .
```

### NixOS Module

The flake includes a NixOS module for easy deployment. Add this to your NixOS configuration:

```nix
{
  inputs.wichtel-loser.url = "github:Vcele/wichtel_loser";

  outputs = { self, nixpkgs, wichtel-loser, ... }: {
    nixosConfigurations.myhost = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        wichtel-loser.nixosModules.x86_64-linux.default
        {
          services.wichtel-loser = {
            enable = true;
            port = 8080;
            address = "127.0.0.1";
          };
        }
      ];
    };
  };
}
```

### Available Flake Outputs

| Output | Description |
|--------|-------------|
| `packages.<system>.default` | The main wichtel_loser package |
| `packages.<system>.wichtel_loser` | Alias for the default package |
| `apps.<system>.default` | Run the application directly |
| `devShells.<system>.default` | Development environment with Rust toolchain |
| `nixosModules.<system>.default` | NixOS module for deployment |

## Manual Build (without Nix)

If you prefer not to use Nix:

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build and run
cargo run --release
```

## License

MIT