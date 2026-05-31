local M = {}

local config = require("stynx.config")
local job = require("stynx.job")
local window = require("stynx.window")

function M.get_visual_selection()
  local _, ls, _ = unpack(vim.fn.getpos("'<"))
  local _, le, _ = unpack(vim.fn.getpos("'>"))
  local lines = vim.api.nvim_buf_get_lines(0, ls - 1, le, false)
  if #lines == 0 then
    return nil
  end
  return table.concat(lines, "\n")
end

function M.get_file_context()
  local buf = vim.api.nvim_get_current_buf()
  local fname = vim.api.nvim_buf_get_name(buf)
  local ft = vim.bo[buf].filetype
  local lines = vim.api.nvim_buf_get_lines(buf, 0, -1, false)
  local content = table.concat(lines, "\n")
  local tag = ft ~= "" and ft or "text"
  local path = fname ~= "" and fname or "unnamed"
  return string.format("```%s\n// %s\n%s\n```", tag, path, content)
end

function M.build_prompt(instruction, with_context)
  local parts = {}
  if with_context then
    local sel = M.get_visual_selection()
    if sel and sel ~= "" then
      table.insert(parts, "Selected code:\n```\n" .. sel .. "\n```\n")
    else
      table.insert(parts, M.get_file_context() .. "\n")
    end
  end
  table.insert(parts, instruction)
  return table.concat(parts, "\n")
end

function M.handle_response(output, buf, win)
  local parsed = nil
  local ok, res = pcall(vim.json.decode, output)
  if ok then
    parsed = res.response or output
  else
    parsed = output
  end
  local lines = vim.split(parsed, "\n")
  window.write_lines(buf, lines)
  if win then
    window.set_close_keymaps(buf, win)
  end
end

function M.send_prompt(prompt)
  if not prompt or prompt == "" then
    vim.notify("stynx: no prompt provided", vim.log.levels.WARN)
    return
  end

  local full = M.build_prompt(prompt, true)
  local buf, win = window.open_result()
  window.write_lines(buf, { "stynx is thinking..." })

  job.run(
    full,
    nil,
    function(output)
      M.handle_response(output, buf, win)
    end,
    function(err)
      window.write_lines(buf, vim.split("error: " .. err, "\n"))
      vim.notify("stynx: " .. err, vim.log.levels.ERROR)
    end
  )
end

function M.explain()
  M.send_prompt(M.build_prompt("Explain this code. What does it do, and what patterns or techniques does it use?", true))
end

function M.fix()
  M.send_prompt(M.build_prompt("Find and fix any bugs, logic errors, or issues in this code. Show the fixes as unified diffs.", true))
end

function M.refactor()
  M.send_prompt(M.build_prompt("Refactor this code. Improve structure, naming, and readability without changing behavior. Show the changes as unified diffs.", true))
end

function M.open_chat()
  local buf = window.open_chat()
  vim.api.nvim_buf_set_keymap(buf, "n", "<CR>", "", {
    callback = function()
      M.send_chat_message()
    end,
  })
  vim.api.nvim_buf_set_keymap(buf, "i", "<CR>", "<Esc>", {
    callback = function()
      M.send_chat_message()
    end,
  })
  vim.cmd("startinsert")
end

function M.send_chat_message()
  local buf = vim.api.nvim_get_current_buf()
  local prompt_lines = vim.api.nvim_buf_get_lines(buf, 3, -1, false)
  local prompt = table.concat(prompt_lines, "\n"):match("^%s*(.-)%s*$")

  if not prompt or prompt == "" then
    return
  end

  vim.api.nvim_buf_set_lines(buf, -1, -1, false, { "", "---", "stynx is thinking..." })
  vim.bo[buf].modifiable = false

  job.run(
    M.build_prompt(prompt, false),
    nil,
    function(output)
      vim.bo[buf].modifiable = true
      local parsed = nil
      local ok, res = pcall(vim.json.decode, output)
      if ok then
        parsed = res.response or output
      else
        parsed = output
      end
      window.append_lines(buf, vim.split(parsed, "\n"))
      window.append_lines(buf, { "", "---", "", "" })
      vim.cmd("normal! G")
      vim.cmd("startinsert")
    end,
    function(err)
      vim.bo[buf].modifiable = true
      window.append_lines(buf, { "error: " .. err, "", "---", "" })
      vim.notify("stynx: " .. err, vim.log.levels.ERROR)
      vim.cmd("startinsert")
    end
  )
end

return M
