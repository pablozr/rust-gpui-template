# Starting a new project

1. Create the repository from this template (GitHub "Use this template"), or clone it and run `git init` again.
2. Rename it. The working tree must be clean:

   ```powershell
   powershell -File tools/rename.ps1 -Name acme-notes -DisplayName "Acme Notes" -Identifier com.example.acmenotes
   ```

   Add `-DryRun` to list what would change. The script renames the crates and folders, the display name, the `MYAPP_` environment variables and the identifier, drops the template block of the README, and removes itself and this file.
3. Run `./tools/check.ps1` (or `tools/check.sh`).
4. Before the first release: set the identifier and the licence holder, add icons under `[package.metadata.packager] icons` in `apps/<name>/Cargo.toml`, and check the licence. Embedding an icon in the `.exe` needs a `build.rs`, which Smart App Control can block on Windows, so do it only through an ADR.
5. Make the first commit.
