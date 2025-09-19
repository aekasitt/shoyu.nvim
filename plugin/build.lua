-- ~~/plugin/build.lua --
-- Shoyu - Beautiful code snippets in Neovim is easy, let me Shoyu!
-- Maintainer: Sitt Guruvanich <aekazitt+github@gmail.com>
-- License: MIT

local Job = require('plenary.job')
if vim.fn.executable('cargo') == 0 then
  vim.schedule(function()
    vim.notify('Building Shoyu shared objects targeting your workstation\'s architecture', vim.log.levels.INFO)
  end)
  Job:new({
    args = { 'build', '--release' },
    command = 'cargo',
    on_exit = function(_, return_val)
      if return_val ~= 0 then
        vim.schedule(function()
          vim.notify('Unable to build Shoyu executable', vim.log.levels.ERROR)
        end)
      end
    end,
    on_stderr = function(_, data)
    end,
  }):start()
end
