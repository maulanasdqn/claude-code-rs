local M = {}

local config = require("stynx.config")

function M.open_result()
  local ui = config.options.float_win
  local w = math.floor(vim.o.columns * ui.width)
  local h = math.floor(vim.o.lines * ui.height)
  local row = math.floor((vim.o.lines - h) / 2)
  local col = math.floor((vim.o.columns - w) / 2)

  local buf = vim.api.nvim_create_buf(false, true)
  vim.bo[buf].bufhidden = "wipe"
  vim.bo[buf].filetype = "markdown"

  local win = vim.api.nvim_open_win(buf, true, {
    relative = "editor",
    width = w,
    height = h,
    row = row,
    col = col,
    style = "minimal",
    border = ui.border,
    title = ui.title,
    title_pos = ui.title_pos,
  })
  vim.wo[win].wrap = true
  vim.wo[win].linebreak = true

  return buf, win
end

function M.open_chat()
  local opts = config.options.chat
  vim.cmd(opts.split_cmd)
  vim.cmd("resize " .. opts.win_height)

  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_win_set_buf(0, buf)
  vim.bo[buf].bufhidden = "wipe"
  vim.bo[buf].filetype = "stynx-chat"
  vim.bo[buf].buftype = "acwrite"
  vim.bo[buf].modifiable = true

  vim.api.nvim_buf_set_lines(buf, 0, -1, false, {
    "╭──────────────────────────────────────────────╮",
    "│  stynx chat - type a message and press <CR>  │",
    "│  :q  to close                                │",
    "╰──────────────────────────────────────────────╯",
    "",
  })

  return buf
end

function M.set_close_keymaps(buf, win)
  vim.api.nvim_buf_set_keymap(buf, "n", "q", "", {
    callback = function()
      vim.api.nvim_win_close(win, true)
    end,
    nowait = true,
  })
  vim.api.nvim_buf_set_keymap(buf, "n", "<Esc>", "", {
    callback = function()
      vim.api.nvim_win_close(win, true)
    end,
    nowait = true,
  })
end

function M.write_lines(buf, lines)
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  vim.bo[buf].modified = false
end

function M.append_lines(buf, lines)
  vim.api.nvim_buf_set_lines(buf, -1, -1, false, lines)
end

return M
