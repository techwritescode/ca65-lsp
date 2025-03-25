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
