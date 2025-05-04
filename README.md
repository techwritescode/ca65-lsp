# CA65 Assembly Language Server
ca65-lsp is a language server, parser, and semantic analyzer for the [CA65](https://cc65.github.io/doc/ca65.html) assembly dialect. It is part of ongoing efforts to improve tooling for the 6502 processor family.

> Note: This project is in active development. Some features may not work fully, others at all. Please watch/star the project to stay up to date!

## Quick Start

TBD

## Editor Setup

- [VSCode](https://github.com/simonhochrein/ca65-code)
- [Zed](https://github.com/simonhochrein/ca65-zed)
- Neovim
  ```lua
  -- init.lua
  require 'lspconfig.configs'.ca65 = {
  	default_config = {
		cmd = { "/path/to/ca65-lsp" },
		filetypes = { "s", "asm" },
		root_dir = require 'lspconfig'.util.root_pattern('nes.toml')
	}
  }
  require 'lspconfig'.ca65.setup{}
  ```
- [Helix](https://helix-editor.com/)
  - if it doesn't already exist, create a [`languages.toml` file](https://docs.helix-editor.com/languages.html#languagestoml-files). I put mine in my [config directory](https://docs.helix-editor.com/configuration.html) which for me (on Windows 11) is `~/AppData/Roaming/helix/`
  - add the following to it:
    ```toml
    # languages.toml
    [language-server.ca65-lsp]
    command = "ca65-lsp"
    
    [[language]]
    name = "ca65"
    scope = "source.s"
    comment-tokens = ";"
    file-types = [ "ca65", "s", "asm" ]
    language-servers = [ "ca65-lsp" ]
    
    [[grammar]]
    name = "ca65"
    source = { git = "https://github.com/simonhochrein/tree-sitter-ca65", rev = "9e73befb5c3c6852f905964c22740c9605b03af8" }
    ```
  - TODO: I believe in order to get syntax highlighting to work you need to implement [queries](https://docs.helix-editor.com/guides/adding_languages.html#queries)
