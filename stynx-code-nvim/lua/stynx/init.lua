local M = {}

M.config = require("stynx.config")
M.job = require("stynx.job")
M.chat = require("stynx.chat")
M.window = require("stynx.window")

function M.setup(opts)
  M.config.setup(opts)
end

return M
