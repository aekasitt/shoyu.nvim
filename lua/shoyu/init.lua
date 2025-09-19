-- ~~/lua/shoyu/init.lua --
-- Shoyu - Beautiful code snippets in Neovim is easy, let me Shoyu!
-- Maintainer: Sitt Guruvanich <aekazitt+github@gmail.com>
-- License: MIT

local M = { lib = nil }
local bit = require('bit')
local ffi = require('ffi')

ffi.cdef[[
  char* generate_snippet_image(
    const char* code,
    const char* language,
    const char* theme,
    const char* config_json
  );
  void free_string(char* s);
  char* get_available_themes(void);
  int is_language_supported(const char* language);
]]

local defaults = {
  width = 1200,
  height = nil,
  padding = 64,
  line_height = .8,
  font_size = 18,
  font_family = 'Fira Code',
  background_color = '#1e1e1e',
  window_controls = true,
  window_title = nil,
  line_numbers = false,
  drop_shadow = true,
  border_radius = 8,
  export_size = 2.0,
  theme = 'dracula',
  output_dir = vim.fn.expand('~/Pictures/shoyu'),
  auto_open = true,
  panel_padding = 80,
  gradient_backdrop = true,
  noise_effect = true,
}
local config = {}

-- Get the plugin directory path
local function get_plugin_dir()
  local source = debug.getinfo(1, 'S').source
  if source:sub(1, 1) == '@' then
    source = source:sub(2)
  end
  -- Go up two levels: from lua/shoyu/init.lua to plugin root
  return vim.fn.fnamemodify(source, ':h:h:h')
end

-- Load the shared library
function M.load_library()
  if M.lib then
    return M.lib
  end
  local file_names = {
    'libshoyu.so',     -- Linux
    'shoyu.dll',       -- Windows
    'libshoyu.dylib',  -- macOS
  }
  local plugin_dir = get_plugin_dir()
  for _, shared_object in ipairs(file_names) do
    local lib_path = plugin_dir .. '/target/release/' .. shared_object
    if vim.fn.filereadable(lib_path) == 1 then
      M.lib = ffi.load(lib_path)
      return M.lib
    end
  end

  error(string.format(
    'Could not load shoyu shared library from plugin directory: %s\n' ..
    'Make sure to build the plugin with: cargo build --release\n' ..
    'Searched paths:\n' ..
    '  - %s/target/release/\n' ..
    '  - %s/target/debug/',
    plugin_dir, plugin_dir, plugin_dir
  ))
end

-- Generate image output
function M.generate_image(opts)
  opts = opts or {}
  if not M.lib then
    M.load_library()
  end

  -- Get current buffer content
  local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)
  local code = table.concat(lines, '\n')
  if code == '' then
    vim.notify('Buffer is empty', vim.log.levels.WARN)
    return
  end

  -- Determine language
  local filetype = opts.language or vim.bo.filetype
  if filetype == '' then
    filetype = 'text'
  end

  -- Check if language is supported
  local lang_cstr = ffi.new('char[?]', #filetype + 1, filetype)
  if M.lib.is_language_supported(lang_cstr) == 0 then
    vim.notify(string.format("Language '%s' is not supported", filetype), vim.log.levels.WARN)
    filetype = 'text'
  end

  -- Merge config with options
  local render_config = vim.tbl_deep_extend('force', config, opts)
  local config_json = vim.json.encode(render_config)

  -- Generate image
  local code_cstr = ffi.new('char[?]', #code + 1, code)
  local lang_cstr = ffi.new('char[?]', #filetype + 1, filetype)
  local theme_cstr = ffi.new('char[?]', #render_config.theme + 1, render_config.theme)
  local config_cstr = ffi.new('char[?]', #config_json + 1, config_json)
  local result = M.lib.generate_snippet_image(code_cstr, lang_cstr, theme_cstr, config_cstr)
  if result == nil then
    vim.notify('Failed to generate image', vim.log.levels.ERROR)
    return
  end

  local image_data = ffi.string(result)
  M.lib.free_string(result)

  -- Save image
  local filename = opts.filename or M.generate_filename(filetype)
  local filepath = config.output_dir .. '/' .. filename
  M.save_image_data(image_data, filepath)
  vim.notify(string.format('Image saved to: %s', filepath), vim.log.levels.INFO)
  if config.auto_open then
    M.open_image(filepath)
  end
  return filepath
end

-- Check if there's an active visual selection
function M.has_visual_selection()
  local start_pos = vim.fn.getpos("'<")
  local end_pos = vim.fn.getpos("'>")

  -- Check if marks exist and are valid
  if start_pos[2] == 0 or end_pos[2] == 0 then
    return false
  end

  -- Check if selection is meaningful (not just a cursor position)
  if start_pos[2] == end_pos[2] and start_pos[3] == end_pos[3] then
    return false
  end
  return true
end

-- Generate image from line range
function M.generate_from_range(start_line, end_line, opts)
  opts = opts or {}

  -- Get lines from range (0-based line numbers)
  local lines = vim.api.nvim_buf_get_lines(0, start_line - 1, end_line, false)
  if #lines == 0 then
    vim.notify('No lines in range', vim.log.levels.WARN)
    return
  end
  local code = table.concat(lines, '\n')

  -- Temporarily replace buffer content for generation
  local original_lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)
  vim.api.nvim_buf_set_lines(0, 0, -1, false, lines)
  local result = M.generate_image(opts)

  -- Restore original content
  vim.api.nvim_buf_set_lines(0, 0, -1, false, original_lines)
  return result
end

-- Generate image from visual selection
function M.generate_from_selection(opts)
  opts = opts or {}
  -- Get visual selection
  local start_pos = vim.fn.getpos("'<")
  local end_pos = vim.fn.getpos("'>")
  -- Convert to line numbers (1-based)
  local start_line = start_pos[2]
  local end_line = end_pos[2]
  return M.generate_from_range(start_line, end_line, opts)
end

-- Generate image intelligently: use selection if available, otherwise full buffer
function M.generate_smart(opts)
  opts = opts or {}
  if M.has_visual_selection() then
    return M.generate_from_selection(opts)
  else
    return M.generate_image(opts)
  end
end

-- Get available themes
function M.get_themes()
  if not M.lib then
    M.load_library()
  end
  local result = M.lib.get_available_themes()
  if result == nil then
    return {}
  end
  local themes_json = ffi.string(result)
  M.lib.free_string(result)
  return vim.json.decode(themes_json)
end

-- Generate filename
function M.generate_filename(filetype)
  local timestamp = os.date('%Y%m%d_%H%M%S')
  local buffer_name = vim.fn.expand('%:t:r')
  if buffer_name == '' then
    buffer_name = 'snippet'
  end
  return string.format('%s_%s_%s.png', buffer_name, filetype, timestamp)
end

-- Save base64 image data to file
function M.save_image_data(data_url, filepath)
  -- Extract base64 data from data URL
  local base64_data = data_url:match('data:image/png;base64,(.+)')
  if not base64_data then
    error('Invalid image data format')
  end
  -- Decode base64 (simple implementation)
  local binary_data = M.decode_base64(base64_data)
  -- Write to file
  local file = io.open(filepath, 'wb')
  if not file then
    error('Could not open file for writing: ' .. filepath)
  end
  file:write(binary_data)
  file:close()
end

-- Simple base64 decoder
function M.decode_base64(data)
  local b64chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'
  local b64lookup = {}
  for i = 1, #b64chars do
    b64lookup[b64chars:sub(i, i)] = i - 1
  end
  local result = {}
  local padding = data:match('=*$')
  data = data:gsub('=*$', '')
  for i = 1, #data, 4 do
    local chunk = data:sub(i, i + 3)
    local a, b, c, d = chunk:byte(1, 4)
    a = b64lookup[string.char(a)]
    b = b64lookup[string.char(b)]
    c = c and b64lookup[string.char(c)]
    d = d and b64lookup[string.char(d)]
    local combined = bit.lshift(a, 18) + bit.lshift(b, 12) + bit.lshift((c or 0), 6) + (d or 0)
    table.insert(result, string.char(bit.band(bit.rshift(combined, 16), 255)))
    if c then
      table.insert(result, string.char(bit.band(bit.rshift(combined, 8), 255)))
    end
    if d then
      table.insert(result, string.char(bit.band(combined, 255)))
    end
  end
  return table.concat(result)
end

function M.open_image(filepath)
  local cmd
  if vim.fn.has('mac') == 1 then
    cmd = 'open'
  elseif vim.fn.has('unix') == 1 then
    cmd = 'xdg-open'
  elseif vim.fn.has('win32') == 1 then
    cmd = 'start'
  else
    vim.notify('Cannot open image: unsupported platform', vim.log.levels.WARN)
    return
  end
  vim.fn.system(cmd .. ' ' .. vim.fn.shellescape(filepath))
end

-- Setup
function M.setup(_, opts)
  config = vim.tbl_deep_extend('force', defaults, opts or {})
  vim.fn.mkdir(config.output_dir, 'p')
  M.load_library()
  vim.api.nvim_create_user_command('Shoyu', function(args)
    if args.args ~= '' then
      opts.theme = args.args
    end
    if args.range > 0 then
      M.generate_from_range(args.line1, args.line2, opts)
    else
      M.generate_smart(opts)
    end
  end, {
    range = true,
    nargs = '?',
    complete = function()
      return M.get_themes()
    end,
    desc = 'Generate code snippet image (uses range/selection if available, otherwise full buffer)'
  })
  vim.api.nvim_create_user_command('ShoyuThemes', function()
    local themes = M.get_themes()
    vim.notify('Available themes: ' .. table.concat(themes, ', '))
  end, {
    desc = 'List available themes'
  })
end

return M
