-- ~~/plugin/shoyu.lua --
-- Shoyu - Beautiful code snippets in Neovim is easy, let me Shoyu!
-- Maintainer: Sitt Guruvanich <aekazitt+github@gmail.com>
-- License: MIT

-- globals --
vim.g.loaded_shoyu = false

-- checks --
if not pcall(require, 'plenary') then
  return
end
local version = vim.version()
if version.major ~= nil and version.minor ~= nil then
  local built = false
  if version.major == 0 then
    local uv = (version.minor <= 10) and vim.loop or vim.uv
    local os_name = uv.os_uname().sysname
    local source = debug.getinfo(1, 'S').source
    if source:sub(1, 1) == '@' then
      source = source:sub(2)
    end
    local plugin_dir = vim.fn.fnamemodify(source, ':h:h')
    if os_name == 'Darwin' then
      if uv.fs_stat(plugin_dir .. '/target/release/libshoyu.dylib') then
        built = true
      end
    elseif os_name == 'Linux' then
      if uv.fs_stat(plugin_dir .. '/target/release/libshoyu.so') then
        built = true
      end
    elseif os_name == 'Windows' then
      if uv.fs_stat(plugin_dir .. '/target/release/shoyu.dll') then
        built = true
      end
    end
  end
  if not built then
    vim.schedule(function()
      vim.notify('Please build binaries necessary for Shoyu.nvim', vim.log.levels.WARN)
    end)
    return
  end
end

vim.g.loaded_shoyu = true
require('shoyu').setup()
