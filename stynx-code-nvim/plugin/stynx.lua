if vim.fn.has("nvim-0.10") == 0 then
  vim.api.nvim_echo({
    { "stynx.nvim requires Neovim >= 0.10", "ErrorMsg" },
  }, true, {})
  return
end

if vim.g.stynx_loaded then
  return
end
vim.g.stynx_loaded = true

local function setup()
  require("stynx").setup()
end

if not pcall(require, "lazy") then
  vim.schedule(setup)
end

return { setup = setup }
