local M = {}

M.defaults = {
  binary = "stynx",
  max_turns = 10,
  timeout_ms = 120000,
  model = nil,
  effort = nil,
  float_win = {
    width = 0.7,
    height = 0.7,
    border = "rounded",
    title = " stynx ",
    title_pos = "center",
  },
  keymaps = {
    chat = "<leader>sc",
    explain = "<leader>se",
    fix = "<leader>sf",
    refactor = "<leader>sr",
  },
  chat = {
    split_cmd = "belowright split",
    win_height = 15,
  },
}

M.options = {}

function M.setup(opts)
  M.options = vim.tbl_deep_extend("force", M.defaults, opts or {})

  vim.api.nvim_create_user_command("Stynx", function(args)
    require("stynx.chat").send_prompt(args.args)
  end, { nargs = "?", desc = "Send a prompt to stynx" })

  vim.api.nvim_create_user_command("StynxExplain", function()
    require("stynx.chat").explain()
  end, { desc = "Ask stynx to explain the current file/selection" })

  vim.api.nvim_create_user_command("StynxFix", function()
    require("stynx.chat").fix()
  end, { desc = "Ask stynx to fix bugs in the current file/selection" })

  vim.api.nvim_create_user_command("StynxRefactor", function()
    require("stynx.chat").refactor()
  end, { desc = "Ask stynx to refactor the current file/selection" })

  vim.api.nvim_create_user_command("StynxChat", function()
    require("stynx.chat").open_chat()
  end, { desc = "Open a stynx chat buffer" })

  local km = M.options.keymaps
  if km.chat then
    vim.keymap.set("n", km.chat, "<cmd>StynxChat<cr>", { desc = "stynx: chat" })
  end
  if km.explain then
    vim.keymap.set({ "n", "v" }, km.explain, "<cmd>StynxExplain<cr>", { desc = "stynx: explain" })
  end
  if km.fix then
    vim.keymap.set({ "n", "v" }, km.fix, "<cmd>StynxFix<cr>", { desc = "stynx: fix" })
  end
  if km.refactor then
    vim.keymap.set({ "n", "v" }, km.refactor, "<cmd>StynxRefactor<cr>", { desc = "stynx: refactor" })
  end
end

return M
